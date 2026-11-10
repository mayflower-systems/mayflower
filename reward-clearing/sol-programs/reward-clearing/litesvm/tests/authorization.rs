use crate::support::*;
use spl_token_interface::error::TokenError;

// Anchor's stable runtime error codes; the suite intentionally does not link the program.
const UNKNOWN_INSTRUCTION: u32 = 101;
const INVALID_ARGUMENTS: u32 = 102;
const WRITABLE: u32 = 2000;
const HAS_ONE: u32 = 2001;
const SEEDS: u32 = 2006;
const DISCRIMINATOR: u32 = 3002;
const DESERIALIZE: u32 = 3003;
const OWNER: u32 = 3007;
const SIGNER: u32 = 3010;

#[test]
fn administrative_and_bidder_mutations_require_the_recorded_signer() {
    let mut f = Fixture::new();
    let impostor = Keypair::new();
    f.bank.fund(&impostor.pubkey());

    let actions = [
        (f.disable_ix(true), f.admin.pubkey()),
        (f.terms_ix(TERMS), f.admin.pubkey()),
        (f.deposit_ix(1), f.bidder.pubkey()),
        (f.withdraw_ix(1), f.bidder.pubkey()),
        (f.claim_ix(), f.bidder.pubkey()),
    ];
    for (instruction, authority) in actions {
        let mut unsigned = instruction.clone();
        meta_mut(&mut unsigned, authority).is_signer = false;
        f.bank.reject(&[unsigned], &[], ix_error(0, SIGNER));

        let mut forged = instruction;
        meta_mut(&mut forged, authority).pubkey = impostor.pubkey();
        f.bank.reject(&[forged], &[&impostor], ix_error(0, HAS_ONE));
    }
}

#[test]
fn group_and_channel_creation_require_the_organization_authority() {
    let mut f = Fixture::empty();
    f.bank.send(&[f.organization_ix()], &[&f.admin]);

    let mut group = f.group_ix();
    meta_mut(&mut group, f.admin.pubkey()).pubkey = f.bidder.pubkey();
    f.bank.reject(&[group], &[&f.bidder], ix_error(0, HAS_ONE));

    f.bank.send(&[f.group_ix()], &[&f.admin]);
    let mut channel = f.channel_ix();
    meta_mut(&mut channel, f.admin.pubkey()).pubkey = f.bidder.pubkey();
    f.bank
        .reject(&[channel], &[&f.bidder], ix_error(0, HAS_ONE));
}

#[test]
fn malformed_instructions_and_missing_write_privileges_are_rejected() {
    let mut f = Fixture::new();
    let mut unknown = f.disable_ix(true);
    unknown.data[0] ^= 0xff;
    let mut truncated = f.disable_ix(true);
    truncated.data.truncate(cpi::DISCRIMINATOR_LEN);
    let mut readonly = f.disable_ix(true);
    meta_mut(&mut readonly, f.addresses.channel).is_writable = false;

    for (instruction, error) in [
        (unknown, UNKNOWN_INSTRUCTION),
        (truncated, INVALID_ARGUMENTS),
        (readonly, WRITABLE),
    ] {
        f.bank
            .reject(&[instruction], &[&f.admin], ix_error(0, error));
    }
}

#[test]
fn account_owner_discriminator_and_layout_are_validated_before_mutation() {
    let mut f = Fixture::new();
    let address = f.addresses.channel;
    let original = f.bank.account(&address);
    let mut wrong_owner = original.clone();
    wrong_owner.owner = cpi::instructions::SYSTEM_PROGRAM_ID;
    let mut wrong_discriminator = original.clone();
    wrong_discriminator.data[0] ^= 0xff;
    let mut truncated = original;
    // Keep the discriminator intact to exercise field decoding separately.
    truncated.data.truncate(cpi::DISCRIMINATOR_LEN);

    for (account, error) in [
        (wrong_owner, OWNER),
        (wrong_discriminator, DISCRIMINATOR),
        (truncated, DESERIALIZE),
    ] {
        f.bank.svm.set_account(address, account).unwrap();
        f.bank
            .reject(&[f.disable_ix(true)], &[&f.admin], ix_error(0, error));
    }
}

#[test]
fn administration_rejects_broken_ownership_links() {
    let mut f = Fixture::new();
    let a = f.addresses;
    let unrelated = Keypair::new().pubkey();
    let group = f.bank.read::<cpi::ChannelGroup>(&a.group);
    f.bank.overwrite(
        &a.group,
        &cpi::ChannelGroup {
            organization: unrelated,
            ..group
        },
    );
    f.bank
        .reject(&[f.disable_ix(true)], &[&f.admin], ix_error(0, HAS_ONE));

    f.bank.overwrite(&a.group, &group);
    let channel = f.bank.read::<cpi::Channel>(&a.channel);
    f.bank.overwrite(
        &a.channel,
        &cpi::Channel {
            channel_group: unrelated,
            ..channel
        },
    );
    f.bank
        .reject(&[f.terms_ix(TERMS)], &[&f.admin], ix_error(0, HAS_ONE));
}

#[test]
fn creation_rejects_a_substituted_pda() {
    let mut f = Fixture::empty();
    let mut instruction = f.organization_ix();
    meta_mut(&mut instruction, f.addresses.organization).pubkey = Keypair::new().pubkey();
    f.bank
        .reject(&[instruction], &[&f.admin], ix_error(0, SEEDS));
    assert!(f.bank.svm.get_account(&f.addresses.organization).is_none());
}

#[test]
fn token_operations_are_bound_to_the_recorded_escrow() {
    let mut f = Fixture::new();
    let a = f.addresses;
    // All replacements are real token accounts. Their valid layouts must not bypass
    // the stored custody relationship, even before a balance or lifecycle check.
    let cases = [
        (f.deposit_ix(1), a.credits, f.credit_source, false),
        (f.withdraw_ix(1), a.credits, f.credit_source, false),
        (f.claim_ix(), a.reward_pot, f.reward_destination, false),
        (f.fund_ix(1), a.reward_pot, f.reward_destination, true),
    ];
    for (mut instruction, escrow, substitute, admin) in cases {
        meta_mut(&mut instruction, escrow).pubkey = substitute;
        let authority = if admin { &f.admin } else { &f.bidder };
        f.bank
            .reject(&[instruction], &[authority], ix_error(0, HAS_ONE));
    }

    let mut finalize = f.finalize_ix();
    meta_mut(&mut finalize, a.credits).pubkey = f.credit_source;
    f.bank.reject(&[finalize], &[], ix_error(0, HAS_ONE));
}

#[test]
fn reward_funding_enforces_the_token_sources_owner_and_mint() {
    let mut f = Fixture::new();
    let mut wrong_owner = f.fund_ix(1);
    meta_mut(&mut wrong_owner, f.admin.pubkey()).pubkey = f.bidder.pubkey();
    let mut wrong_mint = wrong_owner.clone();
    // The bidder really owns the replacement source; only its mint is wrong.
    meta_mut(&mut wrong_mint, f.reward_source).pubkey = f.credit_source;
    for (instruction, error) in [
        (wrong_owner, TokenError::OwnerMismatch),
        (wrong_mint, TokenError::MintMismatch),
    ] {
        f.bank
            .reject(&[instruction], &[&f.bidder], ix_error(0, error as u32));
    }
}

#[test]
fn incoming_self_transfers_cannot_create_unbacked_credits_or_rewards() {
    let mut f = Fixture::new();
    // Give the actual vaults funds so rejection must reach authority validation,
    // rather than accidentally passing because an empty source lacks tokens.
    f.bank
        .send(&[f.deposit_ix(100), f.fund_ix(250)], &[&f.bidder, &f.admin]);
    let mut deposit = f.deposit_ix(1);
    meta_mut(&mut deposit, f.credit_source).pubkey = f.addresses.credits;
    let mut fund = f.fund_ix(1);
    meta_mut(&mut fund, f.reward_source).pubkey = f.addresses.reward_pot;
    for (instruction, signer) in [(deposit, &f.bidder), (fund, &f.admin)] {
        f.bank.reject(
            &[instruction],
            &[signer],
            ix_error(0, TokenError::OwnerMismatch as u32),
        );
    }
}

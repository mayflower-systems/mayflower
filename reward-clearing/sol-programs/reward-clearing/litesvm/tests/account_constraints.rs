use solana_account::Account;
use solana_program_pack::Pack;
use spl_token_interface::{native_mint, state::Mint};

use crate::support::*;
use cpi::MayfRewardClearingError as Error;

fn native_mint(bank: &mut Bank) -> Address {
    let address = native_mint::ID;
    // Canonical external genesis state, not an injected program-owned account.
    if bank.svm.get_account(&address).is_none() {
        let mut data = vec![0; Mint::LEN];
        Mint::pack(
            Mint {
                decimals: native_mint::DECIMALS,
                is_initialized: true,
                ..Mint::default()
            },
            &mut data,
        )
        .unwrap();
        bank.svm
            .set_account(
                address,
                Account {
                    lamports: bank.svm.minimum_balance_for_rent_exemption(data.len()),
                    data,
                    owner: cpi::instructions::TOKEN_PROGRAM_ID,
                    ..Account::default()
                },
            )
            .unwrap();
    }
    address
}

#[test]
fn group_creation_rejects_native_clearing_mint_and_rolls_back_allocations() {
    for packed in [false, true] {
        let mut f = Fixture::empty();
        let native = native_mint(&mut f.bank);
        f.addresses = Addresses::new(
            f.admin.pubkey(),
            f.bidder.pubkey(),
            native,
            f.addresses.reward_mint,
        );
        let mut instructions = Vec::new();
        if packed {
            instructions.push(f.organization_ix());
        } else {
            f.bank.send(&[f.organization_ix()], &[&f.admin]);
        }
        instructions.push(f.group_ix());
        f.bank.reject(
            &instructions,
            &[&f.admin],
            ix_error(u8::from(packed), Error::NativeClearingMint as u32),
        );
    }
}

#[test]
fn native_mint_remains_valid_for_rewards() {
    let mut f = Fixture::empty();
    let native = native_mint(&mut f.bank);
    f.addresses = Addresses::new(
        f.admin.pubkey(),
        f.bidder.pubkey(),
        f.addresses.credit_mint,
        native,
    );
    f.bank
        .send(&[f.organization_ix(), f.group_ix()], &[&f.admin]);
    f.bank.send(&[f.channel_ix()], &[&f.admin]);
    assert_eq!(
        f.bank
            .read::<cpi::Channel>(&f.addresses.channel)
            .reward_token_mint,
        native
    );
    for address in [f.addresses.staging, f.addresses.reward_pot] {
        let vault = f.bank.tokens(&address);
        assert!(vault.is_native());
        assert_eq!(vault.amount, 0);
    }
}

#[test]
fn withdrawal_rejects_its_own_escrow_before_burn_or_refund() {
    let mut f = Fixture::new();
    f.bank.send(&[f.deposit_ix(100)], &[&f.bidder]);
    // The constraint applies both when settlement only refunds and when a burn
    // would precede the refund. Every supplied account is snapshotted by reject.
    for elapsed in [0, DURATION / 2] {
        f.bank.set_time(START + i64::from(elapsed));
        let mut withdraw = f.withdraw_ix(40);
        meta_mut(&mut withdraw, f.credit_source).pubkey = f.addresses.credits;
        f.bank.reject(
            &[withdraw],
            &[&f.bidder],
            ix_error(0, Error::WithdrawalDestinationIsEscrow as u32),
        );
    }
}

#[test]
fn claim_rejects_its_own_escrow_before_payout_or_close() {
    for rewards in [0, 250] {
        let mut f = Fixture::new();
        f.bank.send(
            &[f.deposit_ix(100), f.fund_ix(rewards)],
            &[&f.bidder, &f.admin],
        );
        f.bank.set_time(START + i64::from(DURATION));
        f.bank.send(&[f.finalize_ix()], &[]);
        let mut claim = f.claim_ix();
        meta_mut(&mut claim, f.reward_destination).pubkey = f.addresses.reward_pot;
        f.bank.reject(
            &[claim],
            &[&f.bidder],
            ix_error(0, Error::ClaimDestinationIsEscrow as u32),
        );
    }
}

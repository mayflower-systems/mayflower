use solana_program_pack::Pack;
use spl_token_interface::{error::TokenError, state::Account as TokenAccount};

use crate::support::*;
use cpi::MayfWeightedEmitterError as Error;

// Anchor account-validation codes; these tests do not link the emitter program.
const HAS_ONE: u32 = 2001;
const SEEDS: u32 = 2006;
const TOKEN_MINT: u32 = 2014;
const DUPLICATE_MUTABLE: u32 = 2040;
const OWNER: u32 = 3007;
const SIGNER: u32 = 3010;

#[test]
fn user_and_administrator_actions_require_the_recorded_signer() {
    let mut f = Fixture::new(false);
    let impostor = Keypair::new();
    f.bank.fund(&impostor.pubkey());
    for (instruction, authority, forged_error) in [
        (f.deposit_ix(&f.voter, 1), f.voter.authority.pubkey(), SEEDS),
        (
            f.withdraw_ix(&f.voter, 1),
            f.voter.authority.pubkey(),
            SEEDS,
        ),
        (f.settle_ix(), f.admin.pubkey(), HAS_ONE),
        (f.decay_ix(0), f.admin.pubkey(), HAS_ONE),
    ] {
        let mut unsigned = instruction.clone();
        meta_mut(&mut unsigned, authority).is_signer = false;
        f.bank.reject(&[unsigned], &[], ix_error(0, SIGNER));
        let mut forged = instruction;
        meta_mut(&mut forged, authority).pubkey = impostor.pubkey();
        f.bank
            .reject(&[forged], &[&impostor], ix_error(0, forged_error));
    }
}

#[test]
fn gauge_creation_requires_the_emitter_authority_and_rolls_back_rent() {
    let mut f = Fixture::new(false);
    let gauge = Keypair::new();
    let mut instruction = f.gauge_ix(&gauge);
    meta_mut(&mut instruction, f.admin.pubkey()).pubkey = f.voter.authority.pubkey();
    f.bank.reject(
        &[instruction],
        &[&f.voter.authority, &gauge],
        ix_error(0, HAS_ONE),
    );
}

#[test]
fn emitter_creation_rejects_an_uncontrolled_or_native_emission_mint() {
    for native in [false, true] {
        let mut f = Fixture::empty(false);
        f.emission_mint = if native {
            f.bank.native_mint()
        } else {
            f.bank.mint(f.admin.pubkey())
        };
        f.bank.reject(
            &[f.emitter_ix()],
            &[&f.admin, &f.emitter],
            ix_error(
                0,
                if native {
                    Error::NativeEmissionMint
                } else {
                    Error::InvalidMintAuthority
                } as u32,
            ),
        );
    }
}

#[test]
fn an_escrow_cannot_be_its_own_deposit_source_or_withdrawal_destination() {
    let mut f = Fixture::new(false);
    f.bank
        .send(&[f.deposit_ix(&f.voter, 100)], &[&f.voter.authority]);
    f.bank.set_time(START + 10);
    for mut instruction in [f.deposit_ix(&f.voter, 1), f.withdraw_ix(&f.voter, 1)] {
        meta_mut(&mut instruction, f.voter.source).pubkey = f.voter.escrow;
        f.bank.reject(
            &[instruction],
            &[&f.voter.authority],
            // Anchor rejects writable aliases before the explicit guard runs.
            ix_error(0, DUPLICATE_MUTABLE),
        );
    }
}

#[test]
fn positions_escrows_and_gauges_cannot_be_substituted() {
    let mut f = Fixture::new(false);
    let other = f.add_voter();
    let mut wrong_position = f.deposit_ix(&f.voter, 1);
    meta_mut(&mut wrong_position, f.voter.position).pubkey = other.position;
    f.bank
        .reject(&[wrong_position], &[&f.voter.authority], ix_error(0, SEEDS));
    let mut wrong_escrow = f.deposit_ix(&f.voter, 1);
    meta_mut(&mut wrong_escrow, f.voter.escrow).pubkey = other.escrow;
    f.bank
        .reject(&[wrong_escrow], &[&f.voter.authority], ix_error(0, HAS_ONE));

    let foreign = Fixture::new(false);
    let foreign_gauge = foreign.gauge.pubkey();
    f.bank
        .svm
        .set_account(foreign_gauge, foreign.bank.account(&foreign_gauge))
        .unwrap();
    let mut wrong_gauge = f.settle_ix();
    meta_mut(&mut wrong_gauge, f.gauge.pubkey()).pubkey = foreign_gauge;
    f.bank
        .reject(&[wrong_gauge], &[&f.admin], ix_error(0, HAS_ONE));
}

#[test]
fn token_mints_and_program_account_owners_are_validated() {
    let mut f = Fixture::new(false);
    let wrong_destination = f.bank.token_account(f.vote_mint, f.admin.pubkey(), 0);
    let mut settle = f.settle_ix();
    meta_mut(&mut settle, f.destination).pubkey = wrong_destination;
    f.bank
        .reject(&[settle], &[&f.admin], ix_error(0, TOKEN_MINT));

    let foreign_source = f.bank.token_account(f.vote_mint, f.admin.pubkey(), 100);
    let mut deposit = f.deposit_ix(&f.voter, 1);
    meta_mut(&mut deposit, f.voter.source).pubkey = foreign_source;
    f.bank.reject(
        &[deposit],
        &[&f.voter.authority],
        ix_error(0, TokenError::OwnerMismatch as u32),
    );

    let mut position = f.bank.account(&f.voter.position);
    position.owner = cpi::instructions::SYSTEM_PROGRAM_ID;
    f.bank.svm.set_account(f.voter.position, position).unwrap();
    f.bank.reject(
        &[f.deposit_ix(&f.voter, 1)],
        &[&f.voter.authority],
        ix_error(0, OWNER),
    );
}

#[test]
fn zero_amounts_and_late_packed_failures_leave_all_state_unchanged() {
    let mut f = Fixture::new(false);
    for instruction in [f.deposit_ix(&f.voter, 0), f.withdraw_ix(&f.voter, 0)] {
        f.bank.reject(
            &[instruction],
            &[&f.voter.authority],
            ix_error(0, Error::ZeroAmount as u32),
        );
    }
    f.bank.reject(
        &[f.deposit_ix(&f.voter, 100), f.withdraw_ix(&f.voter, 101)],
        &[&f.voter.authority],
        ix_error(1, Error::InsufficientDeposit as u32),
    );
}

#[test]
fn insufficient_or_frozen_vote_tokens_roll_back_accounting_and_custody() {
    let mut f = Fixture::new(false);
    f.bank.reject(
        &[f.deposit_ix(&f.voter, USER_TOKENS + 1)],
        &[&f.voter.authority],
        ix_error(0, TokenError::InsufficientFunds as u32),
    );
    f.bank
        .send(&[f.deposit_ix(&f.voter, 100)], &[&f.voter.authority]);
    f.bank.freeze(f.voter.source, f.vote_mint, &f.admin);
    f.bank.set_time(START + 10);
    for instruction in [f.deposit_ix(&f.voter, 1), f.withdraw_ix(&f.voter, 1)] {
        f.bank.reject(
            &[instruction],
            &[&f.voter.authority],
            ix_error(0, TokenError::AccountFrozen as u32),
        );
    }
}

#[test]
fn escrow_backing_is_checked_after_the_transfer_with_atomic_rollback() {
    let mut f = Fixture::new(false);
    f.bank
        .send(&[f.deposit_ix(&f.voter, 100)], &[&f.voter.authority]);

    // Fault injection exercises the final safety check; ordinary instructions
    // never reduce escrow backing independently of recorded principal.
    let mut account = f.bank.account(&f.voter.escrow);
    let mut tokens = f.bank.tokens(&f.voter.escrow);
    tokens.amount = 75;
    TokenAccount::pack(tokens, &mut account.data).unwrap();
    f.bank.svm.set_account(f.voter.escrow, account).unwrap();
    f.bank.set_time(START + 10);

    for (instruction, error) in [
        // The transfer's error must precede the backing postcondition.
        (
            f.withdraw_ix(&f.voter, 100),
            TokenError::InsufficientFunds as u32,
        ),
        // After withdrawing 25, the fresh balance is 50 against 75 recorded.
        // A stale pre-CPI balance of 75 would incorrectly satisfy the check.
        (f.withdraw_ix(&f.voter, 25), Error::EscrowUnderfunded as u32),
        (f.deposit_ix(&f.voter, 25), Error::EscrowUnderfunded as u32),
    ] {
        f.bank
            .reject(&[instruction], &[&f.voter.authority], ix_error(0, error));
    }
}

#[test]
fn failed_minting_preserves_both_pending_emission_and_mint_supply() {
    let mut f = Fixture::new(false);
    f.bank
        .send(&[f.deposit_ix(&f.voter, 100)], &[&f.voter.authority]);
    f.bank.freeze(f.destination, f.emission_mint, &f.admin);
    f.bank.set_time(START + 10);
    f.bank.reject(
        &[f.settle_ix()],
        &[&f.admin],
        ix_error(0, TokenError::AccountFrozen as u32),
    );
}

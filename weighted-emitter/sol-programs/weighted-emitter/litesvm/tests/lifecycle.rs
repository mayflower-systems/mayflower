use spl_token_interface::instruction;

use crate::support::*;

#[test]
fn deposits_and_refunds_track_each_position_and_the_aggregate() {
    for same_mint in [false, true] {
        let mut f = Fixture::new(same_mint);
        let other = f.add_voter();
        f.bank.send(
            &[f.deposit_ix(&f.voter, 80), f.deposit_ix(&other, 20)],
            &[&f.voter.authority, &other.authority],
        );
        f.bank
            .send(&[f.withdraw_ix(&f.voter, 30)], &[&f.voter.authority]);
        let (emitter, gauge, position) = f.state();
        assert_eq!(emitter.total_deposited, 70);
        assert_eq!(gauge.total_deposited, 70);
        assert_eq!(position.deposited, 50);
        assert_eq!(
            f.bank.read::<cpi::VotePosition>(&other.position).deposited,
            20
        );
        assert_eq!(f.bank.tokens(&f.voter.escrow).amount, 50);
        assert_eq!(f.bank.tokens(&other.escrow).amount, 20);
        assert_eq!(f.bank.tokens(&f.voter.source).amount, USER_TOKENS - 50);
        assert_eq!(f.bank.supply(&f.emission_mint), INITIAL_SUPPLY);
    }
}

#[test]
fn vote_changes_preserve_pending_emission_until_authorized_settlement() {
    let mut f = Fixture::new(false);
    f.bank
        .send(&[f.deposit_ix(&f.voter, 100)], &[&f.voter.authority]);
    f.bank.set_time(START + 10);
    f.bank
        .send(&[f.withdraw_ix(&f.voter, 100)], &[&f.voter.authority]);
    let (emitter, gauge, position) = f.state();
    let pending = gauge.total_emitted;
    assert!(pending > 0, "withdrawal must persist a pending allocation");
    assert_eq!(emitter.total_emitted, pending);
    assert_eq!(
        (emitter.total_minted, gauge.total_minted, position.deposited),
        (0, 0, 0)
    );
    assert_eq!(f.bank.supply(&f.emission_mint), INITIAL_SUPPLY);
    f.bank.set_time(START + 20);
    let minting = f.minting_snapshot(f.gauge.pubkey(), f.destination);
    f.bank.send(&[f.settle_ix()], &[&f.admin]);
    assert_eq!(minting.assert_minted(&f.bank), pending);
    let before = Snapshot::capture(
        &f.bank,
        [
            f.emitter.pubkey(),
            f.gauge.pubkey(),
            f.emission_mint,
            f.destination,
        ],
    );
    f.bank.send(&[f.settle_ix()], &[&f.admin]);
    before.assert_unchanged(&f.bank);
}

#[test]
fn authorized_settlement_can_mint_to_an_arbitrary_owner_and_to_a_vote_escrow() {
    let mut f = Fixture::new(true);
    f.bank
        .send(&[f.deposit_ix(&f.voter, 100)], &[&f.voter.authority]);
    f.bank.set_time(START + 1);
    let minting = f.minting_snapshot(f.gauge.pubkey(), f.destination);
    f.bank.send(&[f.settle_ix()], &[&f.admin]);
    minting.assert_minted(&f.bank);
    // A same-mint escrow is a valid destination. Minting there is surplus,
    // not an implicit vote deposit: only deposit_vote can create voting weight.
    f.bank.set_time(START + 2);
    let mut settle = f.settle_ix();
    meta_mut(&mut settle, f.destination).pubkey = f.voter.escrow;
    let minting = f.minting_snapshot(f.gauge.pubkey(), f.voter.escrow);
    f.bank.send(&[settle], &[&f.admin]);
    minting.assert_minted(&f.bank);
    let (emitter, gauge, position) = f.state();
    assert_eq!(
        (
            emitter.total_deposited,
            gauge.total_deposited,
            position.deposited
        ),
        (100, 100, 100)
    );
}

#[test]
fn unsolicited_tokens_do_not_create_withdrawable_principal_or_votes() {
    let mut f = Fixture::new(false);
    f.bank
        .send(&[f.deposit_ix(&f.voter, 100)], &[&f.voter.authority]);
    let donation = instruction::transfer_checked(
        &cpi::instructions::TOKEN_PROGRAM_ID,
        &f.voter.source,
        &f.vote_mint,
        &f.voter.escrow,
        &f.voter.authority.pubkey(),
        &[],
        50,
        6,
    )
    .unwrap();
    f.bank.send(
        &[donation, f.withdraw_ix(&f.voter, 100)],
        &[&f.voter.authority],
    );
    let (emitter, gauge, position) = f.state();
    assert_eq!(
        (
            emitter.total_deposited,
            gauge.total_deposited,
            position.deposited
        ),
        (0, 0, 0)
    );
    assert_eq!(f.bank.tokens(&f.voter.escrow).amount, 50);
    f.bank.reject(
        &[f.withdraw_ix(&f.voter, 1)],
        &[&f.voter.authority],
        ix_error(0, cpi::MayfWeightedEmitterError::InsufficientDeposit as u32),
    );
}

#[test]
fn overflowing_emission_rejects_withdrawal_without_changing_custody() {
    // Known liveness issue, inventoried in ../TODO.md: principal remains locked
    // when checkpointing would exceed u64 emission accounting. This records
    // current atomic rejection, not a resolution of that economic policy.
    let mut f = Fixture::empty(false);
    let create = f.emitter_with_rate_ix(u128::from(u64::MAX) << 64);
    f.bank.send(
        &[create, f.gauge_ix(&f.gauge)],
        &[&f.admin, &f.emitter, &f.gauge],
    );
    f.bank.send(
        &[f.position_ix(&f.voter), f.deposit_ix(&f.voter, 1)],
        &[&f.voter.authority],
    );
    f.bank.set_time(START + 3);
    f.bank.reject(
        &[f.withdraw_ix(&f.voter, 1)],
        &[&f.voter.authority],
        ix_error(0, cpi::MayfWeightedEmitterError::ArithmeticOverflow as u32),
    );
}

#[test]
fn settling_one_gauge_mutates_its_accounts_and_the_shared_mint_totals() {
    let mut f = Fixture::new(false);
    let other_gauge = Keypair::new();
    f.bank
        .send(&[f.gauge_ix(&other_gauge)], &[&f.admin, &other_gauge]);
    let other = f.add_voter_for(other_gauge.pubkey());
    f.bank.send(
        &[f.deposit_ix(&f.voter, 100), f.deposit_ix(&other, 100)],
        &[&f.voter.authority, &other.authority],
    );
    assert_eq!(f.state().0.total_deposited, 200);
    assert_eq!(f.state().1.total_deposited, 100);
    assert_eq!(f.bank.read::<cpi::Gauge>(&other.gauge).total_deposited, 100);

    f.bank.set_time(START + 10);
    let other_before = Snapshot::capture(&f.bank, [other.gauge, other.position, other.escrow]);
    let minting = f.minting_snapshot(f.gauge.pubkey(), f.destination);
    f.bank.send(&[f.settle_ix()], &[&f.admin]);
    minting.assert_minted(&f.bank);
    other_before.assert_unchanged(&f.bank);
    let mut settle_other = f.settle_ix();
    meta_mut(&mut settle_other, f.gauge.pubkey()).pubkey = other.gauge;
    let first_before = Snapshot::capture(
        &f.bank,
        [f.gauge.pubkey(), f.voter.position, f.voter.escrow],
    );
    let minting = f.minting_snapshot(other.gauge, f.destination);
    f.bank.send(&[settle_other], &[&f.admin]);
    minting.assert_minted(&f.bank);
    first_before.assert_unchanged(&f.bank);
}

#[test]
fn native_vote_escrows_move_lamports_and_record_only_deposited_tokens() {
    let mut f = Fixture::empty(false);
    f.vote_mint = f.bank.native_mint();
    f.voter.source = f
        .bank
        .token_account(f.vote_mint, f.voter.authority.pubkey(), USER_TOKENS);
    f.initialize();
    assert!(f.bank.tokens(&f.voter.escrow).is_native());
    let original = f.bank.account(&f.voter.escrow).lamports;
    f.bank.send(
        &[f.deposit_ix(&f.voter, 100), f.withdraw_ix(&f.voter, 40)],
        &[&f.voter.authority],
    );
    assert_eq!(f.bank.tokens(&f.voter.escrow).amount, 60);
    assert_eq!(f.bank.account(&f.voter.escrow).lamports, original + 60);
    assert_eq!(f.state().1.total_deposited, 60);
}

#[test]
fn authorized_decay_update_persists_configuration_without_minting() {
    let mut f = Fixture::new(false);
    let untouched = Snapshot::capture(
        &f.bank,
        [
            f.gauge.pubkey(),
            f.voter.position,
            f.voter.escrow,
            f.emission_mint,
            f.destination,
        ],
    );
    let decay = (1_u128 << 64) / 10;
    f.bank.send(&[f.decay_ix(decay)], &[&f.admin]);
    assert_eq!(f.state().0.annual_decay_rate_q64f64, decay);
    untouched.assert_unchanged(&f.bank);
}

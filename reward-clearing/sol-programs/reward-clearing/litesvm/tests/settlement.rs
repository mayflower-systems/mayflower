use spl_token_interface::error::TokenError;
use spl_token_interface::instruction::TokenInstruction;

use crate::support::*;

fn funded(rewards: u64) -> Fixture {
    let mut f = Fixture::new();
    f.bank.send(
        &[f.deposit_ix(100), f.fund_ix(rewards)],
        &[&f.bidder, &f.admin],
    );
    f
}

fn assert_balances(f: &Fixture, credits: u64, shares: u64, rewards: u64) {
    let a = f.addresses;
    let epoch: cpi::ChannelEpoch = f.bank.read(&a.channel_epoch);
    assert_eq!(epoch.clearing_credits_balance, credits);
    assert_eq!(
        epoch.weighted_shares_balance_q64f64,
        u128::from(shares) << 64
    );
    assert_eq!(epoch.reward_pot_balance, rewards);
    assert_eq!(f.bank.tokens(&a.credits).amount, credits);
    assert_eq!(f.bank.tokens(&a.reward_pot).amount, rewards);
}

#[test]
fn deposits_and_withdrawals_settle_custody_and_reuse_allocated_space() {
    let mut f = Fixture::new();
    let a = f.addresses;
    let unchanged = Snapshot::capture(
        &f.bank,
        [a.organization, a.group, a.channel, a.epoch, a.staging],
    );
    let empty = f.bank.account(&a.position);
    let payer_before = f.bank.account(&f.bank.payer.pubkey()).lamports;

    f.bank.send(&[f.deposit_ix(100)], &[&f.bidder]);
    let position: cpi::BidderPosition = f.bank.read(&a.position);
    assert_eq!(
        position.lots,
        [cpi::BidderLot {
            credits: 100,
            shares_q64f64: 100 << 64
        }]
    );
    let grown = f.bank.account(&a.position);
    assert!(grown.data.len() > empty.data.len());
    assert_eq!(grown.data.len(), position.to_account_data().len());
    f.bank.assert_rent_exempt(&a.position);
    assert_eq!(
        payer_before - f.bank.account(&f.bank.payer.pubkey()).lamports,
        grown.lamports - empty.lamports
    );
    assert_eq!(f.bank.tokens(&f.credit_source).amount, INITIAL_TOKENS - 100);
    assert_eq!(f.bank.supply(&a.credit_mint), INITIAL_TOKENS);
    assert_balances(&f, 100, 100, 0);

    // One exact halfway case verifies burn + refund wiring; curve matrices stay
    // in the domain crate. Removing lots must not shrink or refund storage.
    f.bank.set_time(START + i64::from(DURATION / 2));
    f.bank.send(&[f.withdraw_ix(100)], &[&f.bidder]);
    assert!(
        f.bank
            .read::<cpi::BidderPosition>(&a.position)
            .lots
            .is_empty()
    );
    let withdrawn = f.bank.account(&a.position);
    assert_eq!(
        (withdrawn.data.len(), withdrawn.lamports),
        (grown.data.len(), grown.lamports)
    );
    assert_eq!(f.bank.tokens(&f.credit_source).amount, INITIAL_TOKENS - 50);
    assert_eq!(f.bank.supply(&a.credit_mint), INITIAL_TOKENS - 50);
    assert_balances(&f, 0, 0, 0);

    let payer = Snapshot::capture(&f.bank, [f.bank.payer.pubkey()]);
    f.bank.send(&[f.deposit_ix(50)], &[&f.bidder]);
    let reused = f.bank.account(&a.position);
    assert_eq!(
        (reused.data.len(), reused.lamports),
        (grown.data.len(), grown.lamports)
    );
    payer.assert_unchanged(&f.bank);
    assert_balances(&f, 50, 50, 0);
    unchanged.assert_unchanged(&f.bank);
}

#[test]
fn failed_deposit_rolls_back_rent_top_up_and_reallocation() {
    let mut f = Fixture::new();
    let failure = f.bank.reject(
        &[f.deposit_ix(INITIAL_TOKENS + 1)],
        &[&f.bidder],
        ix_error(0, TokenError::InsufficientFunds as u32),
    );
    // The system transfer completed before SPL rejected the source balance.
    assert!(
        failure
            .meta
            .logs
            .iter()
            .any(|log| log == "Program 11111111111111111111111111111111 success")
    );
    f.bank.send(&[f.deposit_ix(100)], &[&f.bidder]);
    assert_balances(&f, 100, 100, 0);
}

#[test]
fn failed_refund_rolls_back_the_preceding_burn() {
    let mut f = funded(250);
    f.bank.set_time(START + i64::from(DURATION / 2));
    let mut withdraw = f.withdraw_ix(40);
    meta_mut(&mut withdraw, f.credit_source).pubkey = f.reward_destination;
    let failure = f.bank.reject(
        &[withdraw],
        &[&f.bidder],
        ix_error(0, TokenError::MintMismatch as u32),
    );
    let invoked: Vec<_> = failure.meta.inner_instructions[0]
        .iter()
        .map(|inner| TokenInstruction::unpack(&inner.instruction.data).unwrap())
        .collect();
    // Reaching the refund proves that the earlier burn succeeded before rollback.
    assert_eq!(
        invoked,
        [
            TokenInstruction::Burn { amount: 20 },
            TokenInstruction::Transfer { amount: 20 }
        ]
    );
    assert_balances(&f, 100, 100, 250);
}

#[test]
fn permissionless_finalization_burns_then_claim_pays_and_closes_to_bidder() {
    // Zero payout skips the token CPI, but must still retire shares and close.
    for rewards in [0, 250] {
        let mut f = funded(rewards);
        let a = f.addresses;
        f.bank.set_time(START + i64::from(DURATION));
        let position = Snapshot::capture(&f.bank, [a.position]);
        let active: cpi::ChannelEpoch = f.bank.read(&a.channel_epoch);
        f.bank.send(&[f.finalize_ix()], &[]);
        position.assert_unchanged(&f.bank);
        let finalized: cpi::ChannelEpoch = f.bank.read(&a.channel_epoch);
        assert_eq!(
            finalized,
            cpi::ChannelEpoch {
                final_totals: Some(cpi::StoredFinalizedTotals {
                    reward_pot: rewards,
                    weighted_shares_q64f64: 100 << 64,
                }),
                ..active
            }
        );
        assert_eq!(f.bank.supply(&a.credit_mint), INITIAL_TOKENS - 100);
        // Finalization retains the historical credit total but burns physical custody.
        assert_eq!(f.bank.tokens(&a.credits).amount, 0);
        assert_eq!(f.bank.tokens(&a.reward_pot).amount, rewards);

        let rent = f.bank.account(&a.position).lamports;
        let bidder_before = f.bank.account(&f.bidder.pubkey()).lamports;
        let unchanged =
            Snapshot::capture(&f.bank, [a.credit_mint, a.credits, f.bank.payer.pubkey()]);
        f.bank.send(&[f.claim_ix()], &[&f.bidder]);
        assert!(
            f.bank.svm.get_account(&a.position).is_none(),
            "claimed position must close"
        );
        assert_eq!(
            f.bank.account(&f.bidder.pubkey()).lamports,
            bidder_before + rent
        );
        assert_eq!(f.bank.tokens(&f.reward_destination).amount, rewards);
        assert_eq!(
            f.bank.tokens(&f.reward_source).amount,
            INITIAL_TOKENS - rewards
        );
        assert_eq!(f.bank.supply(&a.reward_mint), INITIAL_TOKENS);
        assert_eq!(
            f.bank.read::<cpi::ChannelEpoch>(&a.channel_epoch),
            cpi::ChannelEpoch {
                reward_pot_balance: 0,
                weighted_shares_balance_q64f64: 0,
                ..finalized
            }
        );
        assert_eq!(f.bank.tokens(&a.reward_pot).amount, 0);
        unchanged.assert_unchanged(&f.bank);
        // A new transaction cannot collect again from the closed position.
        const ACCOUNT_NOT_INITIALIZED: u32 = 3012;
        f.bank.reject(
            &[f.claim_ix()],
            &[&f.bidder],
            ix_error(0, ACCOUNT_NOT_INITIALIZED),
        );
    }
}

#[test]
fn failed_finalization_preserves_unfinalized_state_and_supply() {
    let mut f = funded(250);
    f.bank.set_time(START + i64::from(DURATION));
    let mut finalize = f.finalize_ix();
    meta_mut(&mut finalize, f.addresses.credit_mint).pubkey = f.addresses.reward_mint;
    f.bank.reject(
        &[finalize],
        &[],
        ix_error(0, TokenError::MintMismatch as u32),
    );
    assert_eq!(
        f.bank
            .read::<cpi::ChannelEpoch>(&f.addresses.channel_epoch)
            .final_totals,
        None
    );
    f.bank.send(&[f.finalize_ix()], &[]);
    assert_eq!(f.bank.tokens(&f.addresses.credits).amount, 0);
    assert_eq!(
        f.bank.supply(&f.addresses.credit_mint),
        INITIAL_TOKENS - 100
    );
}

#[test]
fn failed_payout_preserves_the_position_and_its_rent_for_retry() {
    let mut f = funded(250);
    f.bank.set_time(START + i64::from(DURATION));
    f.bank.send(&[f.finalize_ix()], &[]);
    let mut claim = f.claim_ix();
    meta_mut(&mut claim, f.reward_destination).pubkey = f.credit_source;
    f.bank.reject(
        &[claim],
        &[&f.bidder],
        ix_error(0, TokenError::MintMismatch as u32),
    );
    f.bank.send(&[f.claim_ix()], &[&f.bidder]);
    assert!(f.bank.svm.get_account(&f.addresses.position).is_none());
    assert_eq!(f.bank.tokens(&f.reward_destination).amount, 250);
}

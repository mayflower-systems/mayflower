//! Apply pure numeric receipts to independent ordinary integer balances.
use mayf_fixed_drivers::I64F64 as Number;
use mayf_fixed_number::{FixedNumber, NonNeg};
use mayf_reward_clearing::*;

fn epoch() -> Epoch<Number> {
    start_epoch(StartEpochArgs {
        window: EpochSchedule::try_new(0, 100)
            .unwrap()
            .window_at(0)
            .unwrap(),
        terms: EpochTerms::try_new(100, Number::ONE, 2).unwrap(),
    })
    .unwrap()
    .epoch
}

#[test]
fn numeric_receipts_conserve_principal_and_reward_independently() {
    let mut principal_available = 1_000_u64;
    let mut principal_held = 0_u64;
    let mut forfeited = 0_u64;
    let mut reward_available = 500_u64;
    let mut reward_held = 0_u64;

    let deposited = deposit(
        DepositState {
            epoch: epoch(),
            position: Position::new(),
            latest_lot: None,
        },
        DepositArgs {
            principal: 400,
            now: 0,
        },
    )
    .unwrap();
    principal_available -= deposited.principal_deposited;
    principal_held += deposited.principal_deposited;
    assert_eq!(deposited.epoch.principal(), principal_held);
    assert_eq!(
        deposited.shares_issued,
        NonNeg::new(Number::checked_from_i128(800).unwrap()).unwrap()
    );

    let withdrawn = withdraw(
        WithdrawState {
            epoch: deposited.epoch,
            position: deposited.position,
            lot: deposited.lot,
        },
        WithdrawArgs {
            principal: 100,
            now: 50,
        },
    )
    .unwrap();
    assert_eq!(withdrawn.principal_returned, 75);
    assert_eq!(withdrawn.principal_forfeited, 25);
    assert_eq!(
        withdrawn.principal_returned + withdrawn.principal_forfeited,
        withdrawn.principal_withdrawn
    );
    principal_held -= withdrawn.principal_withdrawn;
    principal_available += withdrawn.principal_returned;
    forfeited += withdrawn.principal_forfeited;
    assert_eq!(withdrawn.epoch.principal(), principal_held);

    let funded = add_reward(
        AddRewardState {
            epoch: withdrawn.epoch,
        },
        AddRewardArgs {
            amount: reward_available,
            now: 50,
        },
    )
    .unwrap();
    reward_available -= funded.reward_to_epoch;
    reward_held += funded.reward_to_epoch;
    assert_eq!(funded.epoch.unclaimed_reward(), reward_held);

    let finalized = settle(
        SettleState {
            epoch: funded.epoch,
        },
        SettleArgs { now: 100 },
    )
    .unwrap();
    assert_eq!(finalized.principal_forfeited, principal_held);
    forfeited += finalized.principal_forfeited;
    principal_held -= finalized.principal_forfeited;
    assert_eq!(principal_available + principal_held + forfeited, 1_000);

    let claimed = claim(
        ClaimState {
            epoch: finalized.epoch,
            position: withdrawn.position,
        },
        ClaimArgs { now: 100 },
    )
    .unwrap();
    reward_held -= claimed.reward_earned;
    assert_eq!(claimed.reward_earned, 500);
    assert_eq!(claimed.epoch.remaining_reward().unwrap(), reward_held);
    assert_eq!(reward_available + reward_held + claimed.reward_earned, 500);
}

#[test]
fn indivisible_withdrawal_forfeits_one_unit_without_a_fractional_balance() {
    let deposited = deposit(
        DepositState {
            epoch: epoch(),
            position: Position::new(),
            latest_lot: None,
        },
        DepositArgs {
            principal: 1,
            now: 0,
        },
    )
    .unwrap();
    let withdrawn = withdraw(
        WithdrawState {
            epoch: deposited.epoch,
            position: deposited.position,
            lot: deposited.lot,
        },
        WithdrawArgs {
            principal: 1,
            now: 50,
        },
    )
    .unwrap();
    assert_eq!(withdrawn.principal_returned, 0);
    assert_eq!(withdrawn.principal_forfeited, 1);
    assert_eq!(withdrawn.principal_withdrawn, 1);
    assert_eq!(withdrawn.position.principal(), 0);
    assert_eq!(withdrawn.epoch.principal(), 0);
    assert!(withdrawn.lot.is_none());
}

#[test]
fn net_funding_is_exact_and_accepts_no_implicit_fee_or_surplus() {
    let funded = add_reward(
        AddRewardState { epoch: epoch() },
        AddRewardArgs {
            amount: 900,
            now: 0,
        },
    )
    .unwrap();
    assert_eq!(funded.reward_to_epoch, 900);
    assert_eq!(funded.epoch.unclaimed_reward(), 900);
    let finalized = settle(
        SettleState {
            epoch: funded.epoch,
        },
        SettleArgs { now: 100 },
    )
    .unwrap();
    assert_eq!(finalized.principal_forfeited, 0);
    assert_eq!(finalized.principal_forfeited, 0);
    assert_eq!(finalized.epoch.remaining_reward(), Ok(900));
}

#[test]
fn unsupported_funding_is_rejected_before_a_future_claim_can_be_stranded() {
    assert_eq!(
        add_reward(
            AddRewardState { epoch: epoch() },
            AddRewardArgs {
                amount: u64::MAX,
                now: 0
            }
        ),
        Err(RewardClearingError::ArithmeticFailure)
    );
    let supported = i64::MAX as u64;
    let funded = add_reward(
        AddRewardState { epoch: epoch() },
        AddRewardArgs {
            amount: supported,
            now: 0,
        },
    )
    .unwrap();
    assert_eq!(
        add_reward(
            AddRewardState {
                epoch: funded.epoch
            },
            AddRewardArgs { amount: 1, now: 0 }
        ),
        Err(RewardClearingError::ArithmeticFailure)
    );
    assert_eq!(funded.epoch.unclaimed_reward(), supported);
    let finalized = settle(
        SettleState {
            epoch: funded.epoch,
        },
        SettleArgs { now: 100 },
    )
    .unwrap();
    assert_eq!(finalized.epoch.remaining_reward(), Ok(supported));
}

#[test]
fn integer_principal_still_requires_representable_weighted_shares() {
    let initial = Epoch::try_from(EpochSnapshot {
        ramp_bonus_slope: Number::ZERO,
        ..epoch().into()
    })
    .unwrap();
    for principal in [i64::MAX as u64 + 1, u64::MAX] {
        assert_eq!(
            deposit(
                DepositState {
                    epoch: initial,
                    position: Position::new(),
                    latest_lot: None
                },
                DepositArgs { principal, now: 0 }
            ),
            Err(RewardClearingError::ArithmeticFailure)
        );
        assert_eq!(initial.principal(), 0);
    }
}

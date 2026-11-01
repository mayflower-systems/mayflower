//! Integer principal and rewards with precision confined to shares and ratios.
use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::FixedNumber;
use mayf_reward_clearing::*;

fn epoch<N: FixedNumber>() -> Epoch<N> {
    start_epoch(StartEpochArgs {
        window: EpochSchedule::try_new(0, 100)
            .unwrap()
            .window_at(0)
            .unwrap(),
        terms: EpochTerms::try_new(100, N::ZERO, 2).unwrap(),
    })
    .unwrap()
    .epoch
}

#[test]
fn unsigned_driver_handles_full_integer_principal_and_reward_balances() {
    let deposited = deposit(
        DepositState {
            epoch: epoch::<U64F64>(),
            position: Position::new(),
            latest_lot: None,
        },
        DepositArgs {
            principal: u64::MAX,
            now: 0,
        },
    )
    .unwrap();
    assert_eq!(deposited.epoch.principal(), u64::MAX);
    assert_eq!(deposited.position.principal(), u64::MAX);
    assert_eq!(deposited.lot.principal(), u64::MAX);
    assert_eq!(
        deposit(
            DepositState {
                epoch: deposited.epoch,
                position: deposited.position,
                latest_lot: Some(deposited.lot)
            },
            DepositArgs {
                principal: 1,
                now: 0
            }
        ),
        Err(RewardClearingError::ArithmeticFailure)
    );
    assert_eq!(deposited.epoch.principal(), u64::MAX);
    let withdrawn = withdraw(
        WithdrawState {
            epoch: deposited.epoch,
            position: deposited.position,
            lot: deposited.lot,
        },
        WithdrawArgs {
            principal: u64::MAX,
            now: 0,
        },
    )
    .unwrap();
    assert_eq!(withdrawn.principal_returned, u64::MAX);
    assert_eq!(withdrawn.principal_forfeited, 0);
    assert_eq!(withdrawn.epoch.principal(), 0);

    // Evaluate the independent settlement branch from the immutable deposit.
    let funded = add_reward(
        AddRewardState {
            epoch: deposited.epoch,
        },
        AddRewardArgs {
            amount: u64::MAX,
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
    assert_eq!(funded.epoch.unclaimed_reward(), u64::MAX);
    let settled = settle(
        SettleState {
            epoch: funded.epoch,
        },
        SettleArgs { now: 100 },
    )
    .unwrap();
    assert_eq!(settled.principal_forfeited, u64::MAX);
    let claimed = claim(
        ClaimState {
            epoch: settled.epoch,
            position: deposited.position,
        },
        ClaimArgs { now: 100 },
    )
    .unwrap();
    assert_eq!(claimed.reward_earned, u64::MAX);
    assert_eq!(claimed.epoch.remaining_reward(), Ok(0));
    assert!(claimed.epoch.claims_complete());
}

#[test]
fn full_integer_reward_pool_uses_the_same_frozen_ratio_for_each_claim() {
    let alice = deposit(
        DepositState {
            epoch: epoch::<U64F64>(),
            position: Position::new(),
            latest_lot: None,
        },
        DepositArgs {
            principal: 3,
            now: 0,
        },
    )
    .unwrap();
    let bob = deposit(
        DepositState {
            epoch: alice.epoch,
            position: Position::new(),
            latest_lot: None,
        },
        DepositArgs {
            principal: 4,
            now: 0,
        },
    )
    .unwrap();
    let funded = add_reward(
        AddRewardState { epoch: bob.epoch },
        AddRewardArgs {
            amount: u64::MAX,
            now: 0,
        },
    )
    .unwrap();
    let settled = settle(
        SettleState {
            epoch: funded.epoch,
        },
        SettleArgs { now: 100 },
    )
    .unwrap();
    let alice_claim = claim(
        ClaimState {
            epoch: settled.epoch,
            position: alice.position,
        },
        ClaimArgs { now: 100 },
    )
    .unwrap();
    let bob_claim = claim(
        ClaimState {
            epoch: alice_claim.epoch,
            position: bob.position,
        },
        ClaimArgs { now: 100 },
    )
    .unwrap();
    assert_eq!(
        alice_claim.reward_earned,
        (u128::from(u64::MAX) * 3 / 7) as u64
    );
    assert_eq!(
        bob_claim.reward_earned,
        (u128::from(u64::MAX) * 4 / 7) as u64
    );
    assert_eq!(bob_claim.epoch.remaining_reward(), Ok(1));
    assert_eq!(
        u128::from(alice_claim.reward_earned)
            + u128::from(bob_claim.reward_earned)
            + u128::from(bob_claim.epoch.remaining_reward().unwrap()),
        u128::from(u64::MAX)
    );
}

use crate::support;
#[test]
fn coarse_grid_claims_conserve_integer_rewards_without_cross_claim_fraction_carry() {
    use support::Tenths;
    for reward in 1..=17_u64 {
        let mut active = epoch::<Tenths>();
        let mut positions = Vec::new();
        for _ in 0..3 {
            let deposited = deposit(
                DepositState {
                    epoch: active,
                    position: Position::new(),
                    latest_lot: None,
                },
                DepositArgs {
                    principal: 1,
                    now: 0,
                },
            )
            .unwrap();
            active = deposited.epoch;
            positions.push(deposited.position);
        }
        let funded = add_reward(
            AddRewardState { epoch: active },
            AddRewardArgs {
                amount: reward,
                now: 0,
            },
        )
        .unwrap();
        let mut finalized = settle(
            SettleState {
                epoch: funded.epoch,
            },
            SettleArgs { now: 100 },
        )
        .unwrap()
        .epoch;
        let mut paid = 0;
        for position in positions {
            let claimed = claim(
                ClaimState {
                    epoch: finalized,
                    position,
                },
                ClaimArgs { now: 100 },
            )
            .unwrap();
            assert_eq!(claimed.reward_earned, reward / 3);
            paid += claimed.reward_earned;
            finalized = claimed.epoch;
        }
        assert_eq!(finalized.remaining_reward(), Ok(reward % 3));
        assert_eq!(paid + finalized.remaining_reward().unwrap(), reward);
    }
}

#[test]
fn persisted_reward_pools_must_fit_the_driver_before_they_can_be_hydrated() {
    use mayf_fixed_drivers::I64F64;
    let active = EpochSnapshot::from(epoch::<I64F64>());
    let finalized = FinalizedEpochSnapshot::from(
        settle(
            SettleState {
                epoch: epoch::<I64F64>(),
            },
            SettleArgs { now: 100 },
        )
        .unwrap()
        .epoch,
    );
    for unsupported in [i64::MAX as u64 + 1, u64::MAX] {
        assert_eq!(
            Epoch::try_from(EpochSnapshot {
                unclaimed_reward: unsupported,
                ..active
            }),
            Err(RewardClearingError::ArithmeticFailure)
        );
        for remaining in [0, unsupported] {
            assert_eq!(
                FinalizedEpoch::try_from(FinalizedEpochSnapshot {
                    settlement_reward: unsupported,
                    unclaimed_reward: remaining,
                    ..finalized
                }),
                Err(RewardClearingError::ArithmeticFailure)
            );
        }
    }
}

#[test]
fn fractional_entitlement_is_floored_independently_for_each_epoch() {
    let mut earned = 0;
    for start in [0, 100, 200] {
        let initial = start_epoch(StartEpochArgs {
            window: EpochSchedule::try_new(0, 100)
                .unwrap()
                .window_at(start)
                .unwrap(),
            terms: EpochTerms::try_new(100, U64F64::ZERO, 2).unwrap(),
        })
        .unwrap()
        .epoch;
        let alice = deposit(
            DepositState {
                epoch: initial,
                position: Position::new(),
                latest_lot: None,
            },
            DepositArgs {
                principal: 1,
                now: start,
            },
        )
        .unwrap();
        let bob = deposit(
            DepositState {
                epoch: alice.epoch,
                position: Position::new(),
                latest_lot: None,
            },
            DepositArgs {
                principal: 2,
                now: start,
            },
        )
        .unwrap();
        let funded = add_reward(
            AddRewardState { epoch: bob.epoch },
            AddRewardArgs {
                amount: 1,
                now: start,
            },
        )
        .unwrap();
        let settled = settle(
            SettleState {
                epoch: funded.epoch,
            },
            SettleArgs { now: start + 100 },
        )
        .unwrap();
        let first = claim(
            ClaimState {
                epoch: settled.epoch,
                position: alice.position,
            },
            ClaimArgs { now: start + 100 },
        )
        .unwrap();
        let second = claim(
            ClaimState {
                epoch: first.epoch,
                position: bob.position,
            },
            ClaimArgs { now: start + 100 },
        )
        .unwrap();
        earned += first.reward_earned;
        assert_eq!(first.reward_earned, 0);
        assert_eq!(second.reward_earned, 0);
        assert_eq!(second.epoch.remaining_reward(), Ok(1));
    }
    assert_eq!(earned, 0);
}

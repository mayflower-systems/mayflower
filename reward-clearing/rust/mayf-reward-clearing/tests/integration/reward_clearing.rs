use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{FixedNumber, NonNeg};
use mayf_reward_clearing::*;

type Number = I64F64;
type Shares = NonNeg<Number>;
const SCALE: i128 = 1_i128 << 64;
fn number(value: i128) -> Number {
    Number::checked_from_i128(value).unwrap()
}
fn amount(value: i128) -> Shares {
    NonNeg::new(number(value)).unwrap()
}
fn fraction(numerator: i128, denominator: i128) -> Shares {
    NonNeg::new(Number::from_scaled_i128(SCALE * numerator / denominator)).unwrap()
}
fn ramp_slope(value: i128) -> RampBonusSlope<Number> {
    RampBonusSlope::new(number(value)).unwrap()
}
fn penalty_exponent(value: u32) -> ExitPenaltyExponent {
    ExitPenaltyExponent::new(value).unwrap()
}
fn duration(value: u64) -> EpochDuration {
    EpochDuration::new(value).unwrap()
}
fn epoch_terms(duration: u64, slope: i128, exponent: u32) -> EpochTerms<Number> {
    EpochTerms::try_new(duration, number(slope), exponent).unwrap()
}
fn epoch(start: u64) -> Epoch<Number> {
    start_epoch(StartEpochArgs {
        window: EpochSchedule::try_new(0, 100)
            .unwrap()
            .window_at(start)
            .unwrap(),
        terms: epoch_terms(100, 1, 2),
    })
    .unwrap()
    .epoch
}
fn update_epoch(
    epoch: Epoch<Number>,
    update: impl FnOnce(&mut EpochSnapshot<Number>),
) -> Epoch<Number> {
    let mut snapshot = epoch.into();
    update(&mut snapshot);
    Epoch::try_from(snapshot).unwrap()
}
fn finalized_epoch(start: u64, shares: u64, reward: u64) -> FinalizedEpoch<Number> {
    let ended = update_epoch(epoch(start), |s| {
        s.principal = shares;
        s.unclaimed_shares = amount(i128::from(shares));
        s.unclaimed_reward = reward;
    });
    settle(
        SettleState { epoch: ended },
        SettleArgs { now: start + 100 },
    )
    .unwrap()
    .epoch
}
fn update_position(
    position: Position<Number>,
    update: impl FnOnce(&mut PositionSnapshot<Number>),
) -> Position<Number> {
    let mut snapshot = position.into();
    update(&mut snapshot);
    Position::try_from(snapshot).unwrap()
}
fn position_with_shares(shares: u64) -> Position<Number> {
    Position::try_from(PositionSnapshot {
        principal: shares,
        shares: amount(i128::from(shares)),
        claimed: false,
    })
    .unwrap()
}
fn open_lot(
    epoch: Epoch<Number>,
    position: Position<Number>,
    latest_lot: Option<Lot<Number>>,
    principal: u64,
    now: u64,
) -> Result<DepositReceipt<Number>> {
    deposit(
        DepositState {
            epoch,
            position,
            latest_lot,
        },
        DepositArgs { principal, now },
    )
}
#[test]
fn curves_use_the_terms_selected_for_an_epoch() {
    let slope = ramp_slope(1);
    assert_eq!(
        entry_weight::<Number>(0, duration(100), slope),
        Ok(amount(2))
    );
    assert_eq!(
        entry_weight::<Number>(25, duration(100), slope),
        Ok(fraction(7, 4))
    );
    assert_eq!(
        entry_weight::<Number>(50, duration(100), slope),
        Ok(fraction(3, 2))
    );
    assert_eq!(
        entry_weight::<Number>(75, duration(100), slope),
        Ok(fraction(5, 4))
    );
    assert_eq!(
        entry_weight::<Number>(100, duration(100), slope),
        Ok(amount(1))
    );
    assert_eq!(
        entry_weight::<Number>(0, duration(100), ramp_slope(2)),
        Ok(amount(3))
    );
    assert_eq!(
        entry_weight::<Number>(100, duration(100), ramp_slope(2)),
        Ok(amount(1))
    );

    assert_eq!(
        withdrawal_penalty::<Number>(50, duration(100), penalty_exponent(1)),
        Ok(fraction(1, 2))
    );
    assert_eq!(
        withdrawal_penalty::<Number>(50, duration(100), penalty_exponent(2)),
        Ok(fraction(1, 4))
    );
    assert_eq!(
        withdrawal_penalty::<Number>(50, duration(100), penalty_exponent(3)),
        Ok(fraction(1, 8))
    );
    assert_eq!(
        withdrawal_penalty::<Number>(0, duration(100), penalty_exponent(4)),
        Ok(amount(0))
    );
    assert_eq!(
        withdrawal_penalty::<Number>(100, duration(100), penalty_exponent(4)),
        Ok(amount(1))
    );

    assert_eq!(
        EpochDuration::new(0),
        Err(RewardClearingError::InvalidEpochDuration)
    );
    assert_eq!(
        withdrawal_penalty::<Number>(101, duration(100), penalty_exponent(2)),
        Err(RewardClearingError::EpochEnded)
    );
}

#[test]
fn curve_term_brands_reject_values_outside_the_protocol_ranges() {
    assert_eq!(ramp_slope(0).get(), amount(0));
    assert_eq!(ramp_slope(2).get(), amount(2));
    assert_eq!(
        RampBonusSlope::new(number(-1)),
        Err(RewardClearingError::InvalidRampBonusSlope)
    );
    assert_eq!(
        RampBonusSlope::new(number(3)),
        Err(RewardClearingError::InvalidRampBonusSlope)
    );
    assert_eq!(
        RampBonusSlope::new(Number::MAX),
        Err(RewardClearingError::InvalidRampBonusSlope)
    );

    assert_eq!(penalty_exponent(1).get(), 1);
    assert_eq!(penalty_exponent(4).get(), 4);
    assert_eq!(
        ExitPenaltyExponent::new(0),
        Err(RewardClearingError::InvalidExitPenaltyExponent)
    );
    assert_eq!(
        ExitPenaltyExponent::new(5),
        Err(RewardClearingError::InvalidExitPenaltyExponent)
    );

    assert_eq!(
        EpochTerms::<Number>::try_new(0, number(1), 2),
        Err(RewardClearingError::InvalidEpochDuration)
    );
    assert_eq!(
        EpochTerms::<Number>::try_new(100, number(3), 2),
        Err(RewardClearingError::InvalidRampBonusSlope)
    );
    assert_eq!(
        EpochTerms::<Number>::try_new(100, number(1), 5),
        Err(RewardClearingError::InvalidExitPenaltyExponent)
    );
}

#[test]
fn persisted_epoch_snapshots_are_revalidated_during_hydration() {
    let snapshot = EpochSnapshot::from(epoch(1_000));

    assert_eq!(
        Epoch::try_from(EpochSnapshot {
            duration: 0,
            ..snapshot
        }),
        Err(RewardClearingError::InvalidEpochDuration)
    );
    assert_eq!(
        Epoch::try_from(EpochSnapshot {
            ramp_bonus_slope: number(3),
            ..snapshot
        }),
        Err(RewardClearingError::InvalidRampBonusSlope)
    );
    assert_eq!(
        Epoch::try_from(EpochSnapshot {
            exit_penalty_exponent: 5,
            ..snapshot
        }),
        Err(RewardClearingError::InvalidExitPenaltyExponent)
    );
    assert_eq!(
        Epoch::try_from(EpochSnapshot {
            master_start: u64::MAX,
            epoch_counter: 0,
            start: u64::MAX,
            duration: 1,
            ..snapshot
        }),
        Err(RewardClearingError::EpochEndOverflow)
    );
    assert_eq!(
        Epoch::try_from(EpochSnapshot {
            start: snapshot.start + 1,
            ..snapshot
        }),
        Err(RewardClearingError::EpochScheduleMismatch)
    );
}

#[test]
fn persisted_accounting_snapshots_reject_impossible_local_ledgers() {
    let empty_epoch: EpochSnapshot<Number> = epoch(1_000).into();
    assert_eq!(
        Epoch::try_from(EpochSnapshot {
            principal: 1,
            ..empty_epoch
        }),
        Err(RewardClearingError::InvalidEpochAccounting)
    );
    assert_eq!(
        Epoch::try_from(EpochSnapshot {
            unclaimed_shares: amount(1),
            ..empty_epoch
        }),
        Err(RewardClearingError::InvalidEpochAccounting)
    );
    assert_eq!(
        Epoch::try_from(EpochSnapshot {
            principal: 1,
            unclaimed_shares: amount(4),
            ..empty_epoch
        }),
        Err(RewardClearingError::InvalidEpochAccounting)
    );
    assert_eq!(
        Epoch::try_from(EpochSnapshot {
            ramp_bonus_slope: amount(0).get(),
            principal: 1,
            unclaimed_shares: amount(2),
            ..empty_epoch
        }),
        Err(RewardClearingError::InvalidEpochAccounting)
    );

    let empty_position: PositionSnapshot<Number> = Position::new().into();
    assert_eq!(
        Position::try_from(PositionSnapshot {
            principal: 1,
            shares: amount(4),
            ..empty_position
        }),
        Err(RewardClearingError::InvalidPositionAccounting)
    );

    let deposited = open_lot(epoch(1_000), Position::new(), None, 10, 1_000).unwrap();
    let valid_lot: LotSnapshot<Number> = deposited.lot.into();
    assert_eq!(
        Lot::try_from(LotSnapshot {
            entry_weight: fraction(1, 2),
            ..valid_lot
        }),
        Err(RewardClearingError::InvalidLotAccounting)
    );
    assert_eq!(
        Lot::try_from(LotSnapshot {
            shares: amount(21),
            ..valid_lot
        }),
        Err(RewardClearingError::InvalidLotAccounting)
    );
}

#[test]
fn finalized_snapshots_reject_only_remainders_above_settlement_totals() {
    let snapshot = FinalizedEpochSnapshot::from(finalized_epoch(1_000, 10, 100));

    assert_eq!(
        FinalizedEpoch::try_from(FinalizedEpochSnapshot {
            unclaimed_shares: amount(11),
            ..snapshot
        }),
        Err(RewardClearingError::FinalizedAccountingExceedsSettlement)
    );
    let completed = FinalizedEpoch::try_from(FinalizedEpochSnapshot {
        unclaimed_shares: amount(0),
        unclaimed_reward: 1,
        ..snapshot
    })
    .unwrap();
    assert!(completed.claims_complete());
}

#[test]
fn actions_bind_position_and_lot_bounds_to_the_epoch_terms() {
    let zero_slope_epoch = update_epoch(epoch(0), |snapshot| {
        snapshot.ramp_bonus_slope = number(0);
        snapshot.principal = 10;
        snapshot.unclaimed_shares = amount(10);
        snapshot.unclaimed_reward = 10;
    });
    let position = update_position(Position::new(), |snapshot| {
        snapshot.principal = 10;
        snapshot.shares = amount(10);
    });
    let impossible_lot = Lot::try_from(LotSnapshot {
        principal: 1,
        shares: amount(3),
        entry_weight: amount(3),
    })
    .unwrap();

    assert_eq!(
        withdraw(
            WithdrawState {
                epoch: zero_slope_epoch,
                position,
                lot: impossible_lot,
            },
            WithdrawArgs {
                principal: 1,
                now: 0,
            }
        ),
        Err(RewardClearingError::InvalidLotAccounting)
    );

    let finalized = settle(
        SettleState {
            epoch: zero_slope_epoch,
        },
        SettleArgs { now: 100 },
    )
    .unwrap()
    .epoch;
    let impossible_position = update_position(position, |snapshot| {
        snapshot.principal = 1;
        snapshot.shares = amount(3);
    });
    assert_eq!(
        claim(
            ClaimState {
                epoch: finalized,
                position: impossible_position,
            },
            ClaimArgs { now: 100 }
        ),
        Err(RewardClearingError::InvalidPositionAccounting)
    );
}

#[test]
fn epoch_creation_snapshots_duration_and_curve_terms() {
    assert_eq!(
        EpochSchedule::try_new(u64::MAX, 1),
        Err(RewardClearingError::EpochEndOverflow)
    );

    let schedule = EpochSchedule::try_new(1_000, 100).unwrap();
    assert_eq!(
        schedule.window_at(999),
        Err(RewardClearingError::ScheduleNotStarted)
    );
    assert_eq!(schedule.end_of(0).unwrap(), 1_100);
    assert_eq!(schedule.end_of(1).unwrap(), 1_200);
    assert_eq!(
        EpochSchedule::try_new(0, 1)
            .unwrap()
            .window_at(u64::from(u32::MAX) + 1),
        Err(RewardClearingError::EpochCounterOverflow)
    );
    let first_window = schedule.window_at(1_000).unwrap();
    let first = start_epoch::<Number>(StartEpochArgs {
        window: first_window,
        terms: epoch_terms(100, 1, 2),
    })
    .unwrap();

    assert_eq!(first.epoch.master_start(), 1_000);
    assert_eq!(first.epoch.epoch_counter(), 0);
    assert_eq!(first.epoch.start(), 1_000);
    assert!(!first.epoch.is_settled(1_099));
    assert!(first.epoch.is_settled(1_100));
    assert_eq!(
        start_epoch::<Number>(StartEpochArgs {
            window: schedule.window_at(1_100).unwrap(),
            terms: epoch_terms(200, 2, 3),
        }),
        Err(RewardClearingError::EpochDurationMismatch)
    );

    let second = start_epoch::<Number>(StartEpochArgs {
        window: schedule.window_at(1_100).unwrap(),
        terms: epoch_terms(100, 2, 3),
    })
    .unwrap();
    assert_eq!(second.epoch.epoch_counter(), 1);
    assert_eq!(second.epoch.duration().get(), 100);
    assert_eq!(second.epoch.ramp_bonus_slope(), ramp_slope(2));
    assert_eq!(second.epoch.exit_penalty_exponent(), penalty_exponent(3));
    assert!(!second.epoch.is_settled(1_199));
    assert!(second.epoch.is_settled(1_200));
}

#[test]
fn a_successor_epoch_does_not_wait_for_predecessor_finalization() {
    let schedule = EpochSchedule::try_new(1_000, 100).unwrap();
    let first = start_epoch::<Number>(StartEpochArgs {
        window: schedule.window_at(1_000).unwrap(),
        terms: epoch_terms(100, 1, 2),
    })
    .unwrap()
    .epoch;

    // Counter 1 is constructible from the schedule while counter 0 is still
    // unfinalized. Time, not predecessor mutation, decides which action
    // each independent epoch accepts at the boundary.
    let second = start_epoch::<Number>(StartEpochArgs {
        window: schedule.window_at(1_100).unwrap(),
        terms: epoch_terms(100, 1, 2),
    })
    .unwrap()
    .epoch;

    assert!(
        add_reward(
            AddRewardState { epoch: second },
            AddRewardArgs {
                amount: 1,
                now: 1_100,
            }
        )
        .is_ok()
    );
    assert_eq!(
        add_reward(
            AddRewardState { epoch: first },
            AddRewardArgs {
                amount: 1,
                now: 1_100,
            }
        ),
        Err(RewardClearingError::EpochEnded)
    );
    assert!(settle(SettleState { epoch: first }, SettleArgs { now: 1_100 }).is_ok());
}

#[test]
fn active_actions_read_duration_from_the_epoch() {
    let short_epoch = start_epoch(StartEpochArgs {
        window: EpochSchedule::try_new(1_000, 50)
            .unwrap()
            .window_at(1_000)
            .unwrap(),
        terms: epoch_terms(50, 1, 2),
    })
    .unwrap()
    .epoch;
    assert!(
        add_reward(
            AddRewardState { epoch: short_epoch },
            AddRewardArgs {
                amount: 1,
                now: 1_049,
            }
        )
        .is_ok()
    );
    assert_eq!(
        add_reward(
            AddRewardState { epoch: short_epoch },
            AddRewardArgs {
                amount: 1,
                now: 1_050,
            }
        ),
        Err(RewardClearingError::EpochEnded)
    );
}

#[test]
fn withdrawal_conserves_principal_across_the_reference_curve() {
    let mut previous_burn = 0;

    for elapsed in 0..epoch(0).duration().get() {
        let deposited = open_lot(epoch(0), Position::new(), None, 100, 0).unwrap();
        let withdrawn = withdraw(
            WithdrawState {
                epoch: deposited.epoch,
                position: deposited.position,
                lot: deposited.lot,
            },
            WithdrawArgs {
                principal: 100,
                now: elapsed,
            },
        )
        .unwrap();

        assert_eq!(
            withdrawn.principal_returned + withdrawn.principal_forfeited,
            100
        );
        assert!(withdrawn.principal_forfeited >= previous_burn);
        assert_eq!(withdrawn.shares_removed, amount(200));
        assert!(withdrawn.lot.is_none());

        assert_eq!(withdrawn.position.principal(), 0);
        assert_eq!(withdrawn.position.shares(), amount(0));
        assert_eq!(withdrawn.epoch.principal(), 0);
        assert_eq!(withdrawn.epoch.unclaimed_shares(), amount(0));

        previous_burn = withdrawn.principal_forfeited;
    }
}

#[test]
fn withdrawal_receipts_are_integral_at_the_token_boundary() {
    let deposited = open_lot(epoch(0), Position::new(), None, 1, 0).unwrap();
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

    assert_eq!(withdrawn.penalty, fraction(1, 4));
    assert_eq!(withdrawn.principal_returned, 0);
    assert_eq!(withdrawn.principal_forfeited, 1);
    assert!(withdrawn.lot.is_none());
    // A whole base unit cannot be split, so the entire lot is penalized and
    // the plan carries a burn with no return alongside it.
    assert_eq!(withdrawn.principal_returned, 0);
    assert_eq!(withdrawn.principal_forfeited, 1);
}

#[test]
fn a_new_lot_cannot_increase_the_supplied_latest_weight() {
    let first = open_lot(epoch(1_000), Position::new(), None, 100, 1_050).unwrap();

    assert_eq!(
        open_lot(first.epoch, first.position, Some(first.lot), 100, 1_025,),
        Err(RewardClearingError::LotWeightOutOfOrder)
    );
}

#[test]
fn splitting_a_same_time_deposit_does_not_create_shares() {
    let single = open_lot(epoch(1_000), Position::new(), None, 100, 1_025).unwrap();
    let split_first = open_lot(epoch(1_000), Position::new(), None, 40, 1_025).unwrap();
    let split_second = open_lot(
        split_first.epoch,
        split_first.position,
        Some(split_first.lot),
        60,
        1_025,
    )
    .unwrap();

    assert_eq!(single.shares_issued, amount(175));
    assert_eq!(split_second.position.shares(), single.shares_issued);
    // Splitting a deposit splits the movement too, and the halves still sum to
    // the whole: the plan never funds more or less than the principal booked.
    assert_eq!(single.principal_deposited, 100);
    assert_eq!(
        split_first.principal_deposited + split_second.principal_deposited,
        single.principal_deposited
    );
}

#[test]
fn partial_and_full_lot_withdrawals_preserve_the_other_lot() {
    let first = open_lot(epoch(1_000), Position::new(), None, 100, 1_000).unwrap();
    let second = open_lot(first.epoch, first.position, Some(first.lot), 100, 1_050).unwrap();
    assert_eq!(second.position.principal(), 200);
    assert_eq!(second.position.shares(), amount(350));

    let state = WithdrawState {
        epoch: second.epoch,
        position: second.position,
        lot: second.lot,
    };
    assert_eq!(
        withdraw(
            state,
            WithdrawArgs {
                principal: 101,
                now: 1_050
            }
        ),
        Err(RewardClearingError::WithdrawalExceedsLotPrincipal)
    );
    let partial = withdraw(
        state,
        WithdrawArgs {
            principal: 40,
            now: 1_050,
        },
    )
    .unwrap();
    assert_eq!(partial.shares_removed, amount(60));
    assert_eq!(partial.principal_returned, 30);
    assert_eq!(partial.principal_forfeited, 10);
    let remainder = partial.lot.unwrap();
    assert_eq!(remainder.principal(), 60);
    assert_eq!(remainder.shares(), amount(90));
    assert_eq!(partial.position.principal(), 160);
    assert_eq!(partial.position.shares(), amount(290));
    assert_eq!(partial.epoch.principal(), 160);
    assert_eq!(partial.epoch.unclaimed_shares(), amount(290));

    let full = withdraw(
        WithdrawState {
            epoch: partial.epoch,
            position: partial.position,
            lot: remainder,
        },
        WithdrawArgs {
            principal: 60,
            now: 1_050,
        },
    )
    .unwrap();
    assert_eq!(full.shares_removed, amount(90));
    assert!(full.lot.is_none());
    assert_eq!(full.position.principal(), first.lot.principal());
    assert_eq!(full.position.shares(), first.lot.shares());
    assert_eq!(full.epoch.unclaimed_reward(), 0);
}

#[test]
fn active_actions_reject_inconsistent_supplied_numeric_totals() {
    let deposited = open_lot(epoch(1_000), Position::new(), None, 10, 1_000).unwrap();
    assert_eq!(
        open_lot(deposited.epoch, deposited.position, None, 1, 1_010),
        Err(RewardClearingError::InvalidPositionAccounting)
    );
    assert_eq!(
        open_lot(
            deposited.epoch,
            Position::new(),
            Some(deposited.lot),
            1,
            1_010
        ),
        Err(RewardClearingError::InvalidPositionAccounting)
    );
    assert_eq!(
        open_lot(
            epoch(1_000),
            deposited.position,
            Some(deposited.lot),
            1,
            1_010
        ),
        Err(RewardClearingError::PositionExceedsEpoch)
    );
    let small_position = position_with_shares(1);
    assert_eq!(
        withdraw(
            WithdrawState {
                epoch: deposited.epoch,
                position: small_position,
                lot: deposited.lot
            },
            WithdrawArgs {
                principal: 1,
                now: 1_010
            },
        ),
        Err(RewardClearingError::LotExceedsPosition)
    );
    let claimed = update_position(deposited.position, |snapshot| snapshot.claimed = true);
    assert_eq!(
        open_lot(deposited.epoch, claimed, Some(deposited.lot), 1, 1_010),
        Err(RewardClearingError::PositionAlreadyClaimed)
    );
}

#[test]
fn earlier_entry_earns_more_reward_for_equal_principal() {
    let early = open_lot(epoch(1_000), Position::new(), None, 100, 1_000).unwrap();
    let late = open_lot(early.epoch, Position::new(), None, 100, 1_050).unwrap();
    assert_eq!(early.position.shares(), amount(200));
    assert_eq!(late.position.shares(), amount(150));
    let funded = add_reward(
        AddRewardState { epoch: late.epoch },
        AddRewardArgs {
            amount: 350,
            now: 1_050,
        },
    )
    .unwrap();
    let settled = settle(
        SettleState {
            epoch: funded.epoch,
        },
        SettleArgs { now: 1_100 },
    )
    .unwrap();
    let early_claim = claim(
        ClaimState {
            epoch: settled.epoch,
            position: early.position,
        },
        ClaimArgs { now: 1_100 },
    )
    .unwrap();
    let late_claim = claim(
        ClaimState {
            epoch: early_claim.epoch,
            position: late.position,
        },
        ClaimArgs { now: 1_100 },
    )
    .unwrap();
    assert_eq!(early_claim.reward_earned, 200);
    assert_eq!(late_claim.reward_earned, 150);
    assert_eq!(late_claim.epoch.remaining_reward(), Ok(0));
}

#[test]
fn claim_enforces_time_eligibility_and_retires_even_zero_reward_shares() {
    let finalized = finalized_epoch(1_000, 3, 1);
    let eligible = position_with_shares(1);
    for (now, expected) in [
        (999, RewardClearingError::TimestampBeforeEpoch),
        (1_099, RewardClearingError::EpochNotEnded),
    ] {
        assert_eq!(
            claim(
                ClaimState {
                    epoch: finalized,
                    position: eligible
                },
                ClaimArgs { now }
            ),
            Err(expected)
        );
    }
    assert_eq!(
        claim(
            ClaimState {
                epoch: finalized,
                position: position_with_shares(4)
            },
            ClaimArgs { now: 1_100 },
        ),
        Err(RewardClearingError::PositionExceedsEpoch)
    );
    let claimed = claim(
        ClaimState {
            epoch: finalized,
            position: eligible,
        },
        ClaimArgs { now: 1_100 },
    )
    .unwrap();
    assert_eq!(claimed.reward_earned, 0);
    assert_eq!(claimed.shares_claimed, amount(1));
    assert!(claimed.position.is_claimed());
    assert_eq!(claimed.epoch.unclaimed_shares(), amount(2));
    assert_eq!(claimed.epoch.unclaimed_reward(), 1);
    assert_eq!(
        claim(
            ClaimState {
                epoch: claimed.epoch,
                position: claimed.position
            },
            ClaimArgs { now: 1_100 },
        ),
        Err(RewardClearingError::PositionAlreadyClaimed)
    );
    let empty = claim(
        ClaimState {
            epoch: finalized_epoch(1_000, 0, 17),
            position: Position::new(),
        },
        ClaimArgs { now: 1_100 },
    )
    .unwrap();
    assert_eq!(empty.reward_earned, 0);
    assert!(empty.position.is_claimed());
    assert_eq!(empty.epoch.remaining_reward(), Ok(17));
}

#[test]
fn settlement_is_time_derived_and_returns_a_distinct_finalized_typestate() {
    let deposited = open_lot(epoch(1_000), Position::new(), None, 100, 1_000).unwrap();
    assert_eq!(
        settle(
            SettleState {
                epoch: deposited.epoch,
            },
            SettleArgs { now: 1_099 }
        ),
        Err(RewardClearingError::EpochNotEnded)
    );
    let first = settle(
        SettleState {
            epoch: deposited.epoch,
        },
        SettleArgs { now: 1_100 },
    )
    .unwrap();
    assert_eq!(first.principal_forfeited, 100);
    assert!(first.epoch.is_settled(1_100));
    assert_eq!(first.epoch.settlement_shares(), amount(200));
}

#[test]
fn claims_are_order_independent_and_report_the_final_rounding_remainder() {
    let settled = finalized_epoch(1_000, 3, 10);
    let first_position = position_with_shares(1);
    let second_position = position_with_shares(2);

    let first_claim = claim(
        ClaimState {
            epoch: settled,
            position: first_position,
        },
        ClaimArgs { now: 1_100 },
    )
    .unwrap();
    let second_claim = claim(
        ClaimState {
            epoch: first_claim.epoch,
            position: second_position,
        },
        ClaimArgs { now: 1_100 },
    )
    .unwrap();

    let reverse_first = claim(
        ClaimState {
            epoch: settled,
            position: second_position,
        },
        ClaimArgs { now: 1_100 },
    )
    .unwrap();
    let reverse_second = claim(
        ClaimState {
            epoch: reverse_first.epoch,
            position: first_position,
        },
        ClaimArgs { now: 1_100 },
    )
    .unwrap();

    assert_eq!(first_claim.reward_earned, 3);
    assert_eq!(first_claim.epoch.unclaimed_reward(), 7);
    assert_eq!(second_claim.reward_earned, 6);
    assert_eq!(reverse_first.reward_earned, 6);
    assert_eq!(reverse_second.reward_earned, 3);
    assert_eq!(second_claim.epoch.unclaimed_shares(), amount(0));
    assert_eq!(second_claim.epoch.unclaimed_reward(), 1);
    assert_eq!(reverse_second.epoch.unclaimed_reward(), 1);
    assert_eq!(second_claim.epoch.remaining_reward().unwrap(), 1);
}

#[test]
fn a_last_claimant_cannot_capture_low_decimal_rounding_dust() {
    let mut settled = finalized_epoch(1_000, 100, 1);

    for _ in 0..100u8 {
        let claimant = position_with_shares(1);
        let receipt = claim(
            ClaimState {
                epoch: settled,
                position: claimant,
            },
            ClaimArgs { now: 1_100 },
        )
        .unwrap();

        assert_eq!(receipt.reward_earned, 0);
        assert_eq!(receipt.epoch.unclaimed_reward(), 1);
        settled = receipt.epoch;
    }

    assert_eq!(settled.unclaimed_shares(), amount(0));
    assert_eq!(settled.unclaimed_reward(), 1);
    assert_eq!(settled.remaining_reward().unwrap(), 1);
}

#[test]
fn a_finalized_epoch_reports_no_remainder_while_claim_shares_remain() {
    assert_eq!(
        finalized_epoch(1_000, 1, 1).remaining_reward(),
        Err(RewardClearingError::EpochClaimsOutstanding)
    );
}

#[test]
fn zero_amount_actions_still_enforce_the_epoch_boundary() {
    let deposited = open_lot(epoch(1_000), Position::new(), None, 1, 1_000).unwrap();
    assert_eq!(
        open_lot(
            deposited.epoch,
            deposited.position,
            Some(deposited.lot),
            0,
            1_001,
        ),
        Err(RewardClearingError::DepositTooSmall)
    );
    assert_eq!(
        withdraw(
            WithdrawState {
                epoch: deposited.epoch,
                position: deposited.position,
                lot: deposited.lot,
            },
            WithdrawArgs {
                principal: 0,
                now: 1_001,
            }
        ),
        Err(RewardClearingError::WithdrawalTooSmall)
    );
    assert_eq!(
        open_lot(
            deposited.epoch,
            deposited.position,
            Some(deposited.lot),
            0,
            1_100,
        ),
        Err(RewardClearingError::EpochEnded)
    );
    assert_eq!(
        withdraw(
            WithdrawState {
                epoch: deposited.epoch,
                position: deposited.position,
                lot: deposited.lot,
            },
            WithdrawArgs {
                principal: 0,
                now: 1_100,
            }
        ),
        Err(RewardClearingError::EpochEnded)
    );
    assert_eq!(
        add_reward(
            AddRewardState {
                epoch: deposited.epoch,
            },
            AddRewardArgs {
                amount: 0,
                now: 1_100,
            }
        ),
        Err(RewardClearingError::EpochEnded)
    );
}

#[test]
fn maximum_supported_reward_claims_floor_the_frozen_ratio_in_whole_units() {
    let reward = i64::MAX as u64;
    let finalized = finalized_epoch(1_000, 7, reward);
    let alice = claim(
        ClaimState {
            epoch: finalized,
            position: position_with_shares(3),
        },
        ClaimArgs { now: 1_100 },
    )
    .unwrap();
    let bob = claim(
        ClaimState {
            epoch: alice.epoch,
            position: position_with_shares(4),
        },
        ClaimArgs { now: 1_100 },
    )
    .unwrap();
    assert_eq!(alice.reward_earned, (u128::from(reward) * 3 / 7) as u64);
    assert_eq!(bob.reward_earned, (u128::from(reward) * 4 / 7) as u64);
    assert_eq!(
        u128::from(alice.reward_earned)
            + u128::from(bob.reward_earned)
            + u128::from(bob.epoch.unclaimed_reward()),
        u128::from(reward)
    );
    assert!(bob.epoch.unclaimed_reward() < 2);
    assert!(bob.epoch.claims_complete());
}

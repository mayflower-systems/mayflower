//! Compare hydration bounds to an unbounded integer oracle on the production grid.

use dashu::integer::IBig;
use mayf_fixed_drivers::I64F64 as Number;
use mayf_fixed_number::NonNeg;
use mayf_reward_clearing::RewardClearingError as E;
use mayf_reward_clearing::*;

const ONE: i128 = 1_i128 << 64;

fn shares(raw: i128) -> NonNeg<Number> {
    NonNeg::new(Number::from_scaled_i128(raw)).unwrap()
}

#[test]
fn hydration_preserves_exact_share_bounds_even_when_the_upper_bound_overflows() {
    // Principal is indivisible, while fractional share-bound edges remain
    // observable all the way to the signed driver's largest supported balance.
    for principal in [
        1_u64,
        2,
        3,
        (i64::MAX as u64) / 3,
        (i64::MAX as u64) / 3 + 1,
        (i64::MAX as u64) / 2,
        (i64::MAX as u64) / 2 + 1,
        i64::MAX as u64,
    ] {
        let principal_raw = i128::from(principal) * ONE;
        for weight in [ONE, ONE + 1, ONE * 3 / 2, ONE * 2, ONE * 3] {
            let maximum = IBig::from(principal) * IBig::from(weight);
            let edge = i128::try_from(maximum.clone()).unwrap_or(i128::MAX);
            for share_raw in [
                principal_raw - 1,
                principal_raw,
                edge - 1,
                edge,
                edge.saturating_add(1),
            ] {
                let valid = share_raw >= principal_raw && IBig::from(share_raw) <= maximum;
                let lot = Lot::try_from(LotSnapshot {
                    principal,
                    shares: shares(share_raw),
                    entry_weight: shares(weight),
                });
                assert_eq!(
                    lot.err(),
                    (!valid).then_some(E::InvalidLotAccounting),
                    "lot: principal={principal}, weight={weight}, shares={share_raw}"
                );

                let epoch = Epoch::try_from(EpochSnapshot {
                    master_start: 0,
                    epoch_counter: 0,
                    start: 0,
                    duration: 100,
                    ramp_bonus_slope: Number::from_scaled_i128(weight - ONE),
                    exit_penalty_exponent: 2,
                    principal,
                    unclaimed_shares: shares(share_raw),
                    unclaimed_reward: 0,
                });
                assert_eq!(
                    epoch.err(),
                    (!valid).then_some(E::InvalidEpochAccounting),
                    "epoch: principal={principal}, weight={weight}, shares={share_raw}"
                );

                if weight == ONE * 3 {
                    let position = Position::try_from(PositionSnapshot {
                        principal,
                        shares: shares(share_raw),
                        claimed: false,
                    });
                    assert_eq!(
                        position.err(),
                        (!valid).then_some(E::InvalidPositionAccounting),
                        "position: principal={principal}, shares={share_raw}"
                    );
                }
            }
        }
    }
}

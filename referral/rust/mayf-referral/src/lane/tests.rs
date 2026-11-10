use mayf_fixed_drivers::{I64Scaled, U192F64};

use super::*;

type Number = U192F64;

fn amount(whole: u64) -> Number {
    Number::from_u64(whole)
}

fn lane(balance: u64, seen: Number) -> Lane<Number> {
    Lane {
        id: LaneId::new(0).unwrap(),
        pool: LanePool {
            balance,
            ..LanePool::default()
        },
        total_earned_seen: seen,
    }
}

#[test]
fn settlement_distributes_only_the_precise_earnings_delta() {
    let half = amount(1).checked_div_floor(amount(2)).unwrap();
    let previous = amount(100).checked_add(half).unwrap();
    let total = amount(103);
    let (next, index) = lane(2, previous).settle(total).unwrap();

    // (103 - 100.5) / 2 = 1.25. No whole-token conversion occurs.
    let expected = amount(5).checked_div_floor(amount(4)).unwrap();
    assert_eq!(index, expected);
    assert_eq!(next.pool.reward_index, expected);
    assert_eq!(next.pool.remainder, Number::ZERO);
    assert_eq!(next.total_earned_seen, total);
}

#[test]
fn settlement_accepts_cumulative_earnings_above_u64() {
    let previous = amount(u64::MAX).checked_add(amount(10)).unwrap();
    let total = previous.checked_add(amount(8)).unwrap();
    let (next, index) = lane(2, previous).settle(total).unwrap();

    assert_eq!(index, amount(4));
    assert_eq!(next.total_earned_seen, total);
}

#[test]
fn repeated_total_leaves_the_lane_unchanged() {
    let mut original = lane(3, amount(10));
    original.pool.reward_index = amount(2);
    // A valid remainder below the next index quantum for this denominator.
    let mut quantum_bytes = [0; 32];
    quantum_bytes[0] = 1;
    original.pool.remainder = Number::from_le_bytes(quantum_bytes);
    let (next, index) = original.clone().settle(amount(10)).unwrap();

    assert_eq!(index, original.pool.reward_index);
    assert_eq!(next, original);
}

#[test]
fn empty_lane_consumes_earnings_without_awarding_them() {
    let mut original = lane(0, amount(12));
    original.pool.unallocated = amount(3);
    let (next, index) = original.settle(amount(20)).unwrap();

    assert_eq!(next.total_earned_seen, amount(20));
    assert_eq!(next.pool.unallocated, amount(11));
    assert_eq!(index, Number::ZERO);
    assert_eq!(next.pool.remainder, Number::ZERO);
}

#[test]
fn settlement_rejects_decreasing_earnings() {
    assert_eq!(
        lane(1, amount(10)).settle(amount(9)).unwrap_err(),
        Error::InvalidEarnings
    );
}

#[test]
fn settlement_rejects_negative_totals_and_corrupt_cursors() {
    type Signed = I64Scaled<1>;
    for (seen, total) in [(0, -1), (-1, 0)] {
        let original = Lane {
            id: LaneId::new(0).unwrap(),
            pool: LanePool::<Signed>::default(),
            total_earned_seen: Signed::from_scaled_i64(seen),
        };
        assert_eq!(
            original.settle(Signed::from_scaled_i64(total)).unwrap_err(),
            Error::InvalidEarnings
        );
    }
}

#[test]
fn settlement_rejects_index_overflow() {
    let mut original = lane(1, Number::ZERO);
    original.pool.reward_index = Number::from_le_bytes([u8::MAX; 32]);

    assert_eq!(
        original.settle(amount(1)).unwrap_err(),
        Error::NumberOutOfRange
    );
}

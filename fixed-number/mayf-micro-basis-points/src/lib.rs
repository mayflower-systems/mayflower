#![no_std]

//! Micro-basis-point wire rates.
//!
//! A micro-basis point is one hundredth of a basis point. One hundred percent
//! is 1,000,000 micro-basis points, one percent is 10,000, and one basis point
//! is 100.
//!
//! Account layouts and instruction arguments persist rates as raw `u32`. This
//! crate owns that scale. It refines a raw value into [`MicroBasisPoints`], a
//! rate proven to sit between zero and one hundred percent, and it converts a
//! rate onto a [`FixedNumber`] grid by dividing by [`ONE`].

use core::num::NonZeroU128;

use mayf_fixed_number::{DirectedRounding, FixedNumber, MathError, MathResult};

/// Micro-basis points in one basis point.
pub const PER_BASIS_POINT: u32 = 100;

/// Micro-basis points in one percent.
pub const PER_PERCENT: u32 = 10_000;

/// Micro-basis points in one whole unit, which is one hundred percent.
pub const ONE: u32 = 1_000_000;

/// [`ONE`] as the divisor every conversion on this scale divides by.
const ONE_DIVISOR: NonZeroU128 = match NonZeroU128::new(ONE as u128) {
    Some(divisor) => divisor,
    None => panic!("one hundred percent is not zero"),
};

/// Divide a raw micro-basis-point value by [`ONE`], rounding as directed.
///
/// This accepts rates above one hundred percent. Multipliers such as the
/// reward-clearing ramp bonus slope share the wire scale but carry a wider
/// range, and the layer that owns the bound validates it. Prefer
/// [`MicroBasisPoints::to_fixed`] for anything that is a percentage.
pub fn raw_to_fixed<N: FixedNumber>(
    micro_basis_points: u32,
    rounding: DirectedRounding,
) -> MathResult<N> {
    N::checked_from_ratio_round(i128::from(micro_basis_points), ONE_DIVISOR, rounding)
}

/// A rate in the closed interval from zero through one hundred percent.
///
/// The inner value is private, so a rate exists only once it has passed the
/// range check. Callers that hold one never re-validate it.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MicroBasisPoints(u32);

impl MicroBasisPoints {
    /// Zero percent.
    pub const ZERO: Self = Self(0);

    /// One hundred percent, the largest rate on this scale.
    pub const ONE_HUNDRED_PERCENT: Self = Self(ONE);

    /// Refine a raw micro-basis-point wire value.
    pub const fn new(micro_basis_points: u32) -> Option<Self> {
        if micro_basis_points <= ONE {
            Some(Self(micro_basis_points))
        } else {
            None
        }
    }

    /// Refine a basis-point wire value.
    pub const fn from_basis_points(basis_points: u32) -> Option<Self> {
        match basis_points.checked_mul(PER_BASIS_POINT) {
            Some(micro_basis_points) => Self::new(micro_basis_points),
            None => None,
        }
    }

    /// Refine a whole-percent value.
    pub const fn from_percent(percent: u32) -> Option<Self> {
        match percent.checked_mul(PER_PERCENT) {
            Some(micro_basis_points) => Self::new(micro_basis_points),
            None => None,
        }
    }

    /// The raw wire value.
    pub const fn get(self) -> u32 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// The remaining share, `one hundred percent - self`.
    pub const fn complement(self) -> Self {
        Self(ONE - self.0)
    }

    /// Divide by [`ONE`] to land on a fixed-point grid.
    pub fn to_fixed<N: FixedNumber>(self, rounding: DirectedRounding) -> MathResult<N> {
        raw_to_fixed(self.0, rounding)
    }

    /// This share of `amount`, rounded down.
    ///
    /// The result never exceeds `amount`, so it cannot overflow.
    pub fn apply_floor(self, amount: u64) -> u64 {
        let numerator = u128::from(amount) * u128::from(self.0);
        (numerator / ONE_DIVISOR.get()) as u64
    }

    /// This share of `amount`, rounded up.
    ///
    /// Rounding a charge up is the Mayflower fee convention: the payer covers
    /// the rounding, never the protocol. The result never exceeds `amount`, so
    /// it cannot overflow.
    pub fn apply_ceil(self, amount: u64) -> u64 {
        let divisor = ONE_DIVISOR.get();
        let numerator = u128::from(amount) * u128::from(self.0);
        (numerator.div_ceil(divisor)) as u64
    }
}

impl TryFrom<u32> for MicroBasisPoints {
    type Error = MathError;

    fn try_from(micro_basis_points: u32) -> MathResult<Self> {
        Self::new(micro_basis_points).ok_or(MathError::Domain)
    }
}

impl From<MicroBasisPoints> for u32 {
    fn from(rate: MicroBasisPoints) -> Self {
        rate.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mayf_fixed_drivers::I64F64;

    fn rate(micro_basis_points: u32) -> MicroBasisPoints {
        MicroBasisPoints::new(micro_basis_points).expect("rate within one hundred percent")
    }

    #[test]
    fn the_scale_constants_agree_with_the_definition() {
        assert_eq!(ONE, 1_00_00_00);
        assert_eq!(PER_PERCENT, 100 * PER_BASIS_POINT);
        assert_eq!(ONE, 100 * PER_PERCENT);
    }

    #[test]
    fn construction_rejects_rates_above_one_hundred_percent() {
        assert_eq!(
            MicroBasisPoints::new(ONE),
            Some(MicroBasisPoints::ONE_HUNDRED_PERCENT)
        );
        assert_eq!(MicroBasisPoints::new(ONE + 1), None);
        assert_eq!(MicroBasisPoints::new(u32::MAX), None);
    }

    #[test]
    fn unit_specific_constructors_share_the_one_scale() {
        assert_eq!(MicroBasisPoints::from_basis_points(10_000), Some(rate(ONE)));
        assert_eq!(MicroBasisPoints::from_basis_points(10_001), None);
        assert_eq!(MicroBasisPoints::from_basis_points(u32::MAX), None);
        assert_eq!(MicroBasisPoints::from_percent(1), Some(rate(10_000)));
        assert_eq!(MicroBasisPoints::from_percent(100), Some(rate(ONE)));
        assert_eq!(MicroBasisPoints::from_percent(101), None);
        assert_eq!(MicroBasisPoints::from_percent(u32::MAX), None);
    }

    #[test]
    fn the_default_rate_is_zero() {
        assert_eq!(MicroBasisPoints::default(), MicroBasisPoints::ZERO);
        assert!(MicroBasisPoints::default().is_zero());
    }

    #[test]
    fn complement_pairs_sum_to_one_hundred_percent() {
        assert_eq!(rate(250_000).complement(), rate(750_000));
        assert_eq!(
            MicroBasisPoints::ZERO.complement(),
            MicroBasisPoints::ONE_HUNDRED_PERCENT
        );
        assert_eq!(
            MicroBasisPoints::ONE_HUNDRED_PERCENT.complement(),
            MicroBasisPoints::ZERO
        );
    }

    #[test]
    fn conversion_divides_by_one_hundred_percent() {
        let half: I64F64 = rate(500_000)
            .to_fixed(DirectedRounding::Floor)
            .expect("half converts");
        let one = I64F64::ONE;
        assert_eq!(half.checked_add(half).expect("sum"), one);

        let none: I64F64 = MicroBasisPoints::ZERO
            .to_fixed(DirectedRounding::Floor)
            .expect("zero converts");
        assert_eq!(none, I64F64::ZERO);

        let all: I64F64 = MicroBasisPoints::ONE_HUNDRED_PERCENT
            .to_fixed(DirectedRounding::Ceil)
            .expect("one hundred percent converts");
        assert_eq!(all, one);
    }

    #[test]
    fn conversion_rounds_in_the_requested_direction() {
        // One micro-basis point is not representable on a binary grid.
        let down: I64F64 = rate(1)
            .to_fixed(DirectedRounding::Floor)
            .expect("floor converts");
        let up: I64F64 = rate(1)
            .to_fixed(DirectedRounding::Ceil)
            .expect("ceil converts");
        assert!(down < up);
        assert_eq!(down.checked_next_up().expect("next up"), up);
    }

    #[test]
    fn every_rate_converts_inside_the_unit_interval() {
        for micro_basis_points in [0, 1, 500_000, ONE - 1, ONE] {
            for rounding in [DirectedRounding::Floor, DirectedRounding::Ceil] {
                let unit: I64F64 = rate(micro_basis_points)
                    .to_fixed(rounding)
                    .expect("rate stays inside the unit interval");
                assert!(unit >= I64F64::ZERO && unit <= I64F64::ONE);
            }
        }
    }

    #[test]
    fn applying_a_rate_rounds_in_the_named_direction() {
        assert_eq!(rate(500_000).apply_floor(3), 1);
        assert_eq!(rate(500_000).apply_ceil(3), 2);
        assert_eq!(rate(1).apply_floor(1), 0);
        assert_eq!(rate(1).apply_ceil(1), 1);
    }

    #[test]
    fn the_closed_interval_bounds_apply_without_overflow() {
        assert_eq!(MicroBasisPoints::ZERO.apply_ceil(u64::MAX), 0);
        assert_eq!(MicroBasisPoints::ZERO.apply_floor(u64::MAX), 0);
        assert_eq!(
            MicroBasisPoints::ONE_HUNDRED_PERCENT.apply_ceil(u64::MAX),
            u64::MAX
        );
        assert_eq!(
            MicroBasisPoints::ONE_HUNDRED_PERCENT.apply_floor(u64::MAX),
            u64::MAX
        );
    }

    #[test]
    fn raw_conversion_accepts_multipliers_above_one_hundred_percent() {
        let slope: I64F64 = raw_to_fixed(2 * ONE, DirectedRounding::Floor).expect("two converts");
        assert_eq!(slope, I64F64::checked_from_i128(2).expect("two"));
        assert_eq!(MicroBasisPoints::new(2 * ONE), None);
    }

    #[test]
    fn conversions_to_and_from_the_wire_type_round_trip() {
        assert_eq!(MicroBasisPoints::try_from(ONE), Ok(rate(ONE)));
        assert_eq!(MicroBasisPoints::try_from(ONE + 1), Err(MathError::Domain));
        assert_eq!(u32::from(rate(12_345)), 12_345);
    }
}

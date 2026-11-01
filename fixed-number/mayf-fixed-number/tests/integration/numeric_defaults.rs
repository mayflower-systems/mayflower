use core::num::NonZeroU128;

use mayf_fixed_number::{DirectedRounding, FixedNumber, MathError, MathResult};

/// A small independent rational-grid driver for exercising trait defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Grid<const SCALE: i128>(i128);

fn quotient(numerator: i128, denominator: i128, rounding: DirectedRounding) -> MathResult<i128> {
    if denominator == 0 {
        return Err(MathError::DivisionByZero);
    }
    let value = numerator
        .checked_div(denominator)
        .ok_or(MathError::Overflow)?;
    let remainder = numerator
        .checked_rem(denominator)
        .ok_or(MathError::Overflow)?;
    if remainder == 0 {
        return Ok(value);
    }
    let positive = (numerator > 0) == (denominator > 0);
    match (rounding, positive) {
        (DirectedRounding::Floor, false) => value.checked_sub(1).ok_or(MathError::Overflow),
        (DirectedRounding::Ceil, true) => value.checked_add(1).ok_or(MathError::Overflow),
        _ => Ok(value),
    }
}

impl<const SCALE: i128> FixedNumber for Grid<SCALE> {
    const ZERO: Self = Self(0);
    const ONE: Self = Self(SCALE);
    const QUANTUM: Self = Self(1);

    fn mul_div_floor(
        self,
        mul: Self,
        div: Self,
        _: mayf_fixed_number::driver::Seal,
    ) -> MathResult<mayf_fixed_number::driver::Floored<Self>> {
        if self < Self::ZERO || mul < Self::ZERO || div < Self::ZERO {
            return Err(MathError::Domain);
        }
        if div == Self::ZERO {
            return Err(MathError::DivisionByZero);
        }
        let numerator = self.0.checked_mul(mul.0).ok_or(MathError::Overflow)?;
        let denominator = div.0;
        let raw = numerator / denominator;
        Ok(mayf_fixed_number::driver::Floored {
            value: Self(raw),
            inexact: numerator % denominator != 0,
        })
    }

    fn ratio_floor(
        numerator: u128,
        denominator: NonZeroU128,
        _: mayf_fixed_number::driver::Seal,
    ) -> MathResult<mayf_fixed_number::driver::Floored<Self>> {
        let numerator = numerator
            .checked_mul(SCALE as u128)
            .ok_or(MathError::Overflow)?;
        let raw = numerator / denominator.get();
        Ok(mayf_fixed_number::driver::Floored {
            value: Self(raw.try_into().map_err(|_| MathError::Overflow)?),
            inexact: numerator % denominator.get() != 0,
        })
    }

    fn checked_from_i128(value: i128) -> MathResult<Self> {
        value
            .checked_mul(SCALE)
            .map(Self)
            .ok_or(MathError::Overflow)
    }

    fn checked_add(self, rhs: Self) -> MathResult<Self> {
        self.0
            .checked_add(rhs.0)
            .map(Self)
            .ok_or(MathError::Overflow)
    }

    fn checked_sub(self, rhs: Self) -> MathResult<Self> {
        self.0
            .checked_sub(rhs.0)
            .map(Self)
            .ok_or(MathError::Overflow)
    }

    fn checked_mul_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        let product = self.0.checked_mul(rhs.0).ok_or(MathError::Overflow)?;
        quotient(product, SCALE, rounding).map(Self)
    }

    fn checked_div_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        let numerator = self.0.checked_mul(SCALE).ok_or(MathError::Overflow)?;
        quotient(numerator, rhs.0, rounding).map(Self)
    }

    fn checked_mul_div_round(
        self,
        multiplier: Self,
        divisor: Self,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let numerator = self
            .0
            .checked_mul(multiplier.0)
            .ok_or(MathError::Overflow)?;
        quotient(numerator, divisor.0, rounding).map(Self)
    }

    fn checked_from_ratio_round(
        numerator: i128,
        denominator: NonZeroU128,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let numerator = numerator.checked_mul(SCALE).ok_or(MathError::Overflow)?;
        let denominator =
            i128::try_from(denominator.get()).map_err(|_| MathError::UnsupportedRange)?;
        quotient(numerator, denominator, rounding).map(Self)
    }
}

#[test]
fn unsigned_widening_is_exact_and_checks_the_driver_range() {
    assert_eq!(Grid::<10>::checked_from_u64(0), Ok(Grid(0)));
    assert_eq!(
        Grid::<10>::checked_from_u64(u64::MAX),
        Ok(Grid(i128::from(u64::MAX) * 10))
    );
    assert_eq!(
        Grid::<{ i128::MAX }>::checked_from_u64(2),
        Err(MathError::Overflow)
    );
}

#[test]
fn powers_preserve_the_existing_order_of_intermediate_rounding() {
    // Exact 1.5^6 is 11.390625. Rounding only the final result would give
    // 11.4, whereas rounded exponentiation by squaring gives a looser upper
    // bound after squaring 1.5, then squaring that rounded value.
    assert_eq!(Grid::<10>(15).checked_pow_ceil(6), Ok(Grid(122)));
}

fn compare_powers_with_exact_rationals<const SCALE: i128>() {
    for raw in 0_i128..=40 {
        for exponent in 0..=9 {
            let upper = Grid::<SCALE>(raw).checked_pow_ceil(exponent).unwrap();
            // Compare without division or grid rounding. All integer powers
            // in this oracle fit i128, independently of the tested algorithm.
            let exact_numerator = raw.pow(exponent) * SCALE;
            let exact_denominator = SCALE.pow(exponent);
            assert!(upper.0 * exact_denominator >= exact_numerator);
        }
    }
}

#[test]
fn upward_powers_bound_independent_exact_rationals_on_multiple_grids() {
    compare_powers_with_exact_rationals::<1>();
    compare_powers_with_exact_rationals::<7>();
    compare_powers_with_exact_rationals::<10>();
}

#[test]
fn powers_handle_identities_domain_and_overflow_without_unused_squaring() {
    assert_eq!(Grid::<10>(0).checked_pow_ceil(0), Ok(Grid(10)));
    assert_eq!(Grid::<10>(-10).checked_pow_ceil(0), Ok(Grid(10)));
    assert_eq!(Grid::<10>(0).checked_pow_ceil(u32::MAX), Ok(Grid(0)));
    assert_eq!(Grid::<10>(10).checked_pow_ceil(u32::MAX), Ok(Grid(10)));
    assert_eq!(Grid::<10>(-1).checked_pow_ceil(2), Err(MathError::Domain));
    assert_eq!(
        Grid::<1>(i128::MAX).checked_pow_ceil(1),
        Ok(Grid(i128::MAX))
    );
    assert_eq!(
        Grid::<1>(i128::MAX).checked_pow_ceil(2),
        Err(MathError::Overflow)
    );
}

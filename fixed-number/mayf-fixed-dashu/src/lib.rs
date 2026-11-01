#![no_std]

//! `mayf-fixed-number` numeric adapter backed by `dashu`.
//!
//! [`DashuFixed12`] stores values on Mayflower's shared `1e-12` fixed grid and
//! uses Dashu arbitrary-precision integers/floats for intermediate arithmetic.
//! Every public result is quantized back to the fixed grid before it leaves this
//! crate.

use core::fmt;
use core::num::NonZeroU128;

use dashu::base::{DivRem, Sign, Signed};
use dashu::float::{Context, DBig, FBig, round::mode};
use dashu::integer::IBig;

use mayf_fixed_number::{
    BoundedExponential, Bounds, DirectedRounding, FixedNumber, IntegralRounding, MathError,
    MathResult, SignedFixed,
};

const SCALE: u32 = 12;
const RAW_ONE: i128 = 1_000_000_000_000;
const EXPM1_PRECISION: usize = 48;
const EXPM1_POSITIVE_OVERFLOW_GUARD_RAW: i128 = 61 * RAW_ONE;
const EXPM1_NEGATIVE_QUANTUM_CUTOFF_RAW: i128 = -28 * RAW_ONE;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DashuFixed12(i128);

impl DashuFixed12 {
    pub const SCALE: u32 = SCALE;
    pub const MAX: Self = Self(i128::MAX);
    pub const MIN: Self = Self(i128::MIN);

    /// Wrap an already-scaled `1e-12` grid value.
    pub const fn from_scaled_i128(value: i128) -> Self {
        Self(value)
    }

    /// Return the raw scaled integer. A logical `1.0` is `1_000_000_000_000`.
    pub const fn to_scaled_i128(self) -> i128 {
        self.0
    }

    /// Construct `mantissa * 10^-scale` if it lies on the fixed `1e-12` grid.
    pub fn try_new(mantissa: i128, scale: u32) -> MathResult<Self> {
        if scale <= SCALE {
            let factor = pow10_i128(SCALE - scale)?;
            mantissa
                .checked_mul(factor)
                .map(Self)
                .ok_or(MathError::Overflow)
        } else {
            let divisor = pow10_i128(scale - SCALE)?;
            if mantissa % divisor == 0 {
                Ok(Self(mantissa / divisor))
            } else {
                Err(MathError::Inexact)
            }
        }
    }

    /// Convert this fixed value to an exact Dashu decimal float.
    fn to_dbig(self) -> DBig {
        DBig::from_parts(IBig::from(self.0), -(SCALE as isize))
    }
}

impl fmt::Display for DashuFixed12 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_dbig().fmt(f)
    }
}

impl From<i32> for DashuFixed12 {
    fn from(value: i32) -> Self {
        Self::checked_from_i128(i128::from(value)).expect("i32 always fits in DashuFixed12")
    }
}

impl FixedNumber for DashuFixed12 {
    const ZERO: Self = Self(0);
    const ONE: Self = Self(RAW_ONE);
    const QUANTUM: Self = Self(1);

    fn mul_div_floor(
        self,
        mul: Self,
        div: Self,
        _: mayf_fixed_number::driver::Seal,
    ) -> MathResult<mayf_fixed_number::driver::Floored<Self>> {
        if self.0 < 0 || mul.0 < 0 || div.0 < 0 {
            return Err(MathError::Domain);
        }
        if div.0 == 0 {
            return Err(MathError::DivisionByZero);
        }
        unsigned_floor(IBig::from(self.0) * IBig::from(mul.0), IBig::from(div.0))
    }

    fn ratio_floor(
        numerator: u128,
        denominator: NonZeroU128,
        _: mayf_fixed_number::driver::Seal,
    ) -> MathResult<mayf_fixed_number::driver::Floored<Self>> {
        unsigned_floor(
            IBig::from(numerator) * IBig::from(RAW_ONE),
            IBig::from(denominator.get()),
        )
    }

    fn checked_from_i128(value: i128) -> MathResult<Self> {
        value
            .checked_mul(RAW_ONE)
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
        checked_mul_div_round(self.0, rhs.0, RAW_ONE, rounding).map(Self)
    }

    fn checked_div_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        checked_mul_div_round(self.0, RAW_ONE, rhs.0, rounding).map(Self)
    }

    fn checked_mul_div_round(
        self,
        multiplier: Self,
        divisor: Self,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        checked_mul_div_round(self.0, multiplier.0, divisor.0, rounding).map(Self)
    }

    fn checked_from_ratio_round(
        numerator: i128,
        denominator: NonZeroU128,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let denominator =
            i128::try_from(denominator.get()).map_err(|_| MathError::UnsupportedRange)?;
        checked_mul_div_round(numerator, RAW_ONE, denominator, rounding).map(Self)
    }
}

impl SignedFixed for DashuFixed12 {
    fn checked_neg(self) -> MathResult<Self> {
        self.0.checked_neg().map(Self).ok_or(MathError::Overflow)
    }
}

impl IntegralRounding for DashuFixed12 {
    fn checked_floor_to_u64(self) -> MathResult<u64> {
        if self.0 < 0 {
            return Err(MathError::NegativeToUnsigned);
        }
        u64::try_from(self.0 / RAW_ONE).map_err(|_| MathError::Overflow)
    }
}

impl BoundedExponential for DashuFixed12 {
    fn checked_exp_m1_bounds(self) -> MathResult<Bounds<Self>> {
        if self.is_zero() {
            return Ok(Bounds::exact(Self::ZERO));
        }
        if self.0 >= EXPM1_POSITIVE_OVERFLOW_GUARD_RAW {
            return Err(MathError::Overflow);
        }
        if self.0 <= EXPM1_NEGATIVE_QUANTUM_CUTOFF_RAW {
            return Bounds::new(Self(-RAW_ONE), Self(-RAW_ONE + 1));
        }

        let x = self.to_dbig();
        let lower_context = Context::<mode::Down>::new(EXPM1_PRECISION);
        let upper_context = Context::<mode::Up>::new(EXPM1_PRECISION);

        let lower = lower_context.exp_m1(x.repr()).value();
        let upper = upper_context.exp_m1(x.repr()).value();

        conservative_bounds(lower, upper)
    }
}

fn unsigned_floor(
    numerator: IBig,
    denominator: IBig,
) -> MathResult<mayf_fixed_number::driver::Floored<DashuFixed12>> {
    let (quotient, remainder) = numerator.div_rem(denominator);
    Ok(mayf_fixed_number::driver::Floored {
        value: DashuFixed12(i128::try_from(quotient).map_err(|_| MathError::Overflow)?),
        inexact: !remainder.is_zero(),
    })
}

fn pow10_i128(exponent: u32) -> MathResult<i128> {
    10_i128.checked_pow(exponent).ok_or(MathError::Overflow)
}

fn checked_mul_div_round(
    left: i128,
    right: i128,
    divisor: i128,
    rounding: DirectedRounding,
) -> MathResult<i128> {
    if divisor == 0 {
        return Err(MathError::DivisionByZero);
    }

    let numerator = IBig::from(left) * IBig::from(right);
    let denominator = IBig::from(divisor);
    div_to_raw_round(numerator, denominator, rounding)
}

fn div_to_raw_round(
    mut numerator: IBig,
    mut denominator: IBig,
    rounding: DirectedRounding,
) -> MathResult<i128> {
    if denominator.is_zero() {
        return Err(MathError::DivisionByZero);
    }

    if denominator.is_negative() {
        numerator = -numerator;
        denominator = -denominator;
    }

    let numerator_is_negative = numerator.is_negative();
    let numerator_is_positive = numerator.sign() == Sign::Positive && !numerator.is_zero();
    let (mut quotient, remainder) = numerator.div_rem(denominator);

    if !remainder.is_zero() {
        match rounding {
            DirectedRounding::Floor if numerator_is_negative => {
                quotient -= IBig::ONE;
            }
            DirectedRounding::Ceil if numerator_is_positive => {
                quotient += IBig::ONE;
            }
            _ => {}
        }
    }

    i128::try_from(quotient).map_err(|_| MathError::Overflow)
}

fn fbig_to_raw<R: dashu::float::round::Round>(value: FBig<R, 10>) -> MathResult<i128> {
    let scaled = value * FBig::<R, 10>::from(IBig::from(RAW_ONE));
    let raw = scaled.to_int().value();
    i128::try_from(raw).map_err(|_| MathError::Overflow)
}

fn conservative_bounds(
    lower: FBig<mode::Down, 10>,
    upper: FBig<mode::Up, 10>,
) -> MathResult<Bounds<DashuFixed12>> {
    let lower = DashuFixed12(fbig_to_raw(lower)?);
    let upper = DashuFixed12(fbig_to_raw(upper)?);

    Bounds::new(
        lower.checked_next_down().unwrap_or(lower),
        upper.checked_next_up().unwrap_or(upper),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(mantissa: i128, scale: u32) -> DashuFixed12 {
        DashuFixed12::try_new(mantissa, scale).unwrap()
    }

    #[test]
    fn constants_define_the_dashu_grid() {
        assert_eq!(DashuFixed12::ZERO.to_scaled_i128(), 0);
        assert_eq!(DashuFixed12::ONE.to_scaled_i128(), RAW_ONE);
        assert_eq!(DashuFixed12::QUANTUM.to_scaled_i128(), 1);
    }

    #[test]
    fn floor_to_u64_checks_sign_and_the_rounded_amount_range() {
        assert_eq!(fixed(19, 1).checked_floor_to_u64(), Ok(1));
        assert_eq!(fixed(2, 0).checked_floor_to_u64(), Ok(2));
        assert_eq!(DashuFixed12::ZERO.checked_floor_to_u64(), Ok(0));
        assert_eq!(DashuFixed12::QUANTUM.checked_floor_to_u64(), Ok(0));
        for value in [fixed(-11, 1), DashuFixed12(-1), DashuFixed12::MIN] {
            assert_eq!(
                value.checked_floor_to_u64(),
                Err(MathError::NegativeToUnsigned)
            );
        }
        let maximum = i128::from(u64::MAX) * RAW_ONE;
        assert_eq!(DashuFixed12(maximum).checked_floor_to_u64(), Ok(u64::MAX));
        assert_eq!(
            DashuFixed12(maximum + RAW_ONE - 1).checked_floor_to_u64(),
            Ok(u64::MAX)
        );
        for value in [DashuFixed12(maximum + RAW_ONE), DashuFixed12::MAX] {
            assert_eq!(value.checked_floor_to_u64(), Err(MathError::Overflow));
        }
    }

    #[test]
    fn rejects_values_between_grid_points() {
        assert_eq!(DashuFixed12::try_new(1, SCALE + 1), Err(MathError::Inexact));
    }

    #[test]
    fn from_ratio_uses_directed_quantum_rounding() {
        let denominator = NonZeroU128::new(3).unwrap();

        let lower = DashuFixed12::checked_from_ratio_round(1, denominator, DirectedRounding::Floor)
            .unwrap();
        let upper =
            DashuFixed12::checked_from_ratio_round(1, denominator, DirectedRounding::Ceil).unwrap();

        assert_eq!(lower.to_scaled_i128(), 333_333_333_333);
        assert_eq!(upper.to_scaled_i128(), 333_333_333_334);
        assert_eq!(upper.checked_sub(lower), Ok(DashuFixed12::QUANTUM));
    }

    #[test]
    fn negative_rounding_is_mathematical_floor_and_ceil() {
        let denominator = NonZeroU128::new(3).unwrap();

        let lower =
            DashuFixed12::checked_from_ratio_round(-1, denominator, DirectedRounding::Floor)
                .unwrap();
        let upper = DashuFixed12::checked_from_ratio_round(-1, denominator, DirectedRounding::Ceil)
            .unwrap();

        assert_eq!(lower.to_scaled_i128(), -333_333_333_334);
        assert_eq!(upper.to_scaled_i128(), -333_333_333_333);
        assert_eq!(upper.checked_sub(lower), Ok(DashuFixed12::QUANTUM));
    }

    #[test]
    fn multiplication_uses_directed_rounding() {
        let one_third = DashuFixed12::ONE
            .checked_div_floor(fixed(3, 0))
            .expect("1 / 3");

        let floor = one_third
            .checked_mul_round(one_third, DirectedRounding::Floor)
            .unwrap();
        let ceil = one_third
            .checked_mul_round(one_third, DirectedRounding::Ceil)
            .unwrap();

        assert_eq!(floor.to_scaled_i128(), 111_111_111_110);
        assert_eq!(ceil.to_scaled_i128(), 111_111_111_111);
    }

    #[test]
    fn fused_mul_div_uses_one_rounding_step() {
        let one = DashuFixed12::ONE;
        let three = fixed(3, 0);

        let divided = one.checked_div_floor(three).unwrap();
        let fused = one.checked_mul_div_floor(one, three).unwrap();

        assert_eq!(divided, fused);
    }

    #[test]
    fn dashu_intermediates_allow_large_checked_results() {
        assert_eq!(
            DashuFixed12::MAX.checked_mul_floor(DashuFixed12::ONE),
            Ok(DashuFixed12::MAX)
        );
        assert_eq!(
            DashuFixed12::MAX.checked_mul_div_floor(DashuFixed12::MAX, DashuFixed12::MAX),
            Ok(DashuFixed12::MAX)
        );
    }

    #[test]
    fn checked_arithmetic_reports_failures() {
        assert_eq!(
            DashuFixed12::ONE.checked_div_floor(DashuFixed12::ZERO),
            Err(MathError::DivisionByZero)
        );
        assert_eq!(
            DashuFixed12::MAX.checked_add(DashuFixed12::QUANTUM),
            Err(MathError::Overflow)
        );
        assert_eq!(DashuFixed12::MIN.checked_neg(), Err(MathError::Overflow));
    }

    #[test]
    fn decimal_float_projection_rounds_to_native_grid() {
        let value = DBig::from_parts(IBig::from(1), -13);

        assert_eq!(
            fbig_to_raw(value.clone().with_rounding::<mode::Down>()),
            Ok(0)
        );
        assert_eq!(fbig_to_raw(value.with_rounding::<mode::Up>()), Ok(1));

        let negative = DBig::from_parts(IBig::from(-1), -13);

        assert_eq!(
            fbig_to_raw(negative.clone().with_rounding::<mode::Down>()),
            Ok(-1)
        );
        assert_eq!(fbig_to_raw(negative.with_rounding::<mode::Up>()), Ok(0));
    }

    #[test]
    fn exp_m1_bounds_contain_basic_values() {
        assert_eq!(
            DashuFixed12::ZERO.checked_exp_m1_bounds().unwrap(),
            Bounds::exact(DashuFixed12::ZERO)
        );

        let bounds = DashuFixed12::ONE.checked_exp_m1_bounds().unwrap();
        let expected = DashuFixed12::from_scaled_i128(1_718_281_828_459);

        assert!(bounds.lower() <= expected);
        assert!(expected <= bounds.upper());
    }

    #[test]
    fn exp_m1_bounds_handle_extreme_inputs_without_huge_intermediates() {
        assert_eq!(
            DashuFixed12::from_i32(61).unwrap().checked_exp_m1_bounds(),
            Err(MathError::Overflow)
        );

        let bounds = DashuFixed12::from_i32(-28)
            .unwrap()
            .checked_exp_m1_bounds()
            .unwrap();

        assert_eq!(bounds.lower(), DashuFixed12::from_scaled_i128(-RAW_ONE));
        assert_eq!(bounds.upper(), DashuFixed12::from_scaled_i128(-RAW_ONE + 1));
    }
}

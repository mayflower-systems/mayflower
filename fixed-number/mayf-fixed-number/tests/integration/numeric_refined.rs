use core::num::NonZeroU128;

use mayf_fixed_number::{
    AboveOne, DirectedRounding, FixedNumber, MathError, MathResult, NonNeg, Pos,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct IntFixed(i128);

impl FixedNumber for IntFixed {
    const ZERO: Self = Self(0);
    const ONE: Self = Self(1);
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
        let raw = numerator / denominator.get();
        Ok(mayf_fixed_number::driver::Floored {
            value: Self(raw.try_into().map_err(|_| MathError::Overflow)?),
            inexact: !numerator.is_multiple_of(denominator.get()),
        })
    }

    fn checked_from_i128(value: i128) -> MathResult<Self> {
        Ok(Self(value))
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

    fn checked_mul_round(self, rhs: Self, _rounding: DirectedRounding) -> MathResult<Self> {
        self.0
            .checked_mul(rhs.0)
            .map(Self)
            .ok_or(MathError::Overflow)
    }

    fn checked_div_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        if rhs.0 == 0 {
            return Err(MathError::DivisionByZero);
        }

        let quotient = self.0 / rhs.0;
        let remainder = self.0 % rhs.0;
        if remainder == 0 {
            return Ok(Self(quotient));
        }

        let same_sign = (self.0 >= 0) == (rhs.0 >= 0);
        match (rounding, same_sign) {
            (DirectedRounding::Floor, false) => {
                quotient.checked_sub(1).map(Self).ok_or(MathError::Overflow)
            }
            (DirectedRounding::Ceil, true) => {
                quotient.checked_add(1).map(Self).ok_or(MathError::Overflow)
            }
            _ => Ok(Self(quotient)),
        }
    }

    fn checked_mul_div_round(
        self,
        multiplier: Self,
        divisor: Self,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        self.checked_mul_round(multiplier, rounding)?
            .checked_div_round(divisor, rounding)
    }

    fn checked_from_ratio_round(
        numerator: i128,
        denominator: NonZeroU128,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let denominator =
            i128::try_from(denominator.get()).map_err(|_| MathError::UnsupportedRange)?;
        Self(numerator).checked_div_round(Self(denominator), rounding)
    }
}

fn n(value: i128) -> IntFixed {
    IntFixed(value)
}

#[test]
fn construction_enforces_the_brand() {
    assert!(NonNeg::new(n(0)).is_ok());
    assert_eq!(NonNeg::new(n(-1)), Err(MathError::Domain));
    assert!(Pos::new(n(1)).is_ok());
    assert_eq!(Pos::new(n(0)), Err(MathError::Domain));
    assert!(AboveOne::new(n(2)).is_ok());
    assert_eq!(AboveOne::new(n(1)), Err(MathError::Domain));
}

#[test]
fn widenings_are_infallible_and_narrowings_check() {
    let p = Pos::new(n(5)).unwrap();
    let non_negative: NonNeg<IntFixed> = p.into();
    let raw = non_negative.get();
    assert_eq!(raw, n(5));
    assert_eq!(NonNeg::new(raw).unwrap(), non_negative);
    assert_eq!(NonNeg::new(n(-3)), Err(MathError::Domain));
}

#[test]
fn non_neg_sub_folds_the_underflow_check() {
    let five = NonNeg::new(n(5)).unwrap();
    let three = NonNeg::new(n(3)).unwrap();
    assert_eq!(five.checked_sub(three).map(NonNeg::get), Ok(n(2)));
    assert_eq!(three.checked_sub(five), Err(MathError::Domain));
}

#[test]
fn pos_divisor_cannot_divide_by_zero() {
    let ten = NonNeg::new(n(10)).unwrap();
    let two = Pos::from_i32(2).unwrap();
    assert_eq!(ten.checked_div_floor(two).map(NonNeg::get), Ok(n(5)));
}

#[test]
fn ratio_construction_uses_directed_rounding() {
    let three = NonZeroU128::new(3).unwrap();

    assert_eq!(
        IntFixed::checked_from_ratio_round(10, three, DirectedRounding::Floor),
        Ok(n(3))
    );
    assert_eq!(
        IntFixed::checked_from_ratio_round(10, three, DirectedRounding::Ceil),
        Ok(n(4))
    );
}

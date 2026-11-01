//! Deliberately coarse, signed test driver: one native quantum is 0.1 units.
//! It implements the production numeric trait, with no grid access API.
use core::num::NonZeroU128;
use mayf_fixed_number::{DirectedRounding, FixedNumber, MathError, MathResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tenths(pub i64);

fn rounded(numerator: i128, denominator: i128, mode: DirectedRounding) -> MathResult<Tenths> {
    if denominator == 0 {
        return Err(MathError::DivisionByZero);
    }
    let quotient = numerator
        .checked_div(denominator)
        .ok_or(MathError::Overflow)?;
    let remainder = numerator
        .checked_rem(denominator)
        .ok_or(MathError::Overflow)?;
    let same_sign = (numerator < 0) == (denominator < 0);
    let adjustment = match mode {
        DirectedRounding::Floor if remainder != 0 && !same_sign => -1,
        DirectedRounding::Ceil if remainder != 0 && same_sign => 1,
        _ => 0,
    };
    let raw = quotient
        .checked_add(adjustment)
        .ok_or(MathError::Overflow)?;
    i64::try_from(raw)
        .map(Tenths)
        .map_err(|_| MathError::Overflow)
}

impl FixedNumber for Tenths {
    const ZERO: Self = Self(0);
    const ONE: Self = Self(10);
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
        let numerator = i128::from(self.0)
            .checked_mul(i128::from(mul.0))
            .ok_or(MathError::Overflow)?;
        let denominator = i128::from(div.0);
        let raw = numerator / denominator;
        Ok(mayf_fixed_number::driver::Floored {
            value: Self(raw.try_into().map_err(|_| MathError::Overflow)?),
            inexact: numerator % denominator != 0,
        })
    }

    fn ratio_floor(
        numerator: u128,
        denominator: NonZeroU128,
        _: mayf_fixed_number::driver::Seal,
    ) -> MathResult<mayf_fixed_number::driver::Floored<Self>> {
        let numerator = numerator.checked_mul(10_u128).ok_or(MathError::Overflow)?;
        let raw = numerator / denominator.get();
        Ok(mayf_fixed_number::driver::Floored {
            value: Self(raw.try_into().map_err(|_| MathError::Overflow)?),
            inexact: numerator % denominator.get() != 0,
        })
    }

    fn checked_from_i128(value: i128) -> MathResult<Self> {
        let raw = value.checked_mul(10).ok_or(MathError::Overflow)?;
        i64::try_from(raw)
            .map(Self)
            .map_err(|_| MathError::Overflow)
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

    fn checked_mul_round(self, rhs: Self, mode: DirectedRounding) -> MathResult<Self> {
        rounded(i128::from(self.0) * i128::from(rhs.0), 10, mode)
    }
    fn checked_div_round(self, rhs: Self, mode: DirectedRounding) -> MathResult<Self> {
        rounded(i128::from(self.0) * 10, i128::from(rhs.0), mode)
    }
    fn checked_mul_div_round(
        self,
        multiplier: Self,
        divisor: Self,
        mode: DirectedRounding,
    ) -> MathResult<Self> {
        rounded(
            i128::from(self.0) * i128::from(multiplier.0),
            i128::from(divisor.0),
            mode,
        )
    }
    fn checked_from_ratio_round(
        numerator: i128,
        denominator: NonZeroU128,
        mode: DirectedRounding,
    ) -> MathResult<Self> {
        rounded(
            numerator.checked_mul(10).ok_or(MathError::Overflow)?,
            i128::try_from(denominator.get()).map_err(|_| MathError::Overflow)?,
            mode,
        )
    }
}

impl mayf_fixed_number::IntegralRounding for Tenths {
    fn checked_floor_to_u64(self) -> MathResult<u64> {
        if self.0 < 0 {
            return Err(MathError::NegativeToUnsigned);
        }
        Ok((self.0 / 10) as u64)
    }
}

impl mayf_fixed_number::SignedFixed for Tenths {
    fn checked_neg(self) -> MathResult<Self> {
        self.0.checked_neg().map(Self).ok_or(MathError::Overflow)
    }
}

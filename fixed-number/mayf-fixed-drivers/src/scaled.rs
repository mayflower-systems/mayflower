//! Fixed grids parameterized by the number of raw quanta in one whole unit.

use core::num::NonZeroU128;
use crypto_bigint::{CheckedAdd, CheckedSub, Encoding, NonZero, U256, U512};
use mayf_fixed_number::driver::{Floored, Seal};
use mayf_fixed_number::{
    DirectedRounding, FixedNumber, IntegralRounding, MathError, MathResult, SignedFixed,
    UnsignedFixed,
};

use crate::kernels::unsigned_mul_div_floor;

/// Signed 64-bit storage on the grid `1 / SCALE`.
///
/// `SCALE` counts quanta per whole unit, not decimal places. For example,
/// `I64Scaled<10>` has quantum `0.1`. The scale must be in `1..=i64::MAX`;
/// invalid scales fail constant evaluation when a value is constructed.
/// Products and ratios use wide intermediates and one final directed rounding,
/// including for negative results and full-width `u128` denominators.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct I64Scaled<const SCALE: u64>(i64);

impl<const SCALE: u64> I64Scaled<SCALE> {
    pub const SCALE: u64 = {
        assert!(
            SCALE > 0 && SCALE <= i64::MAX as u64,
            "invalid signed scale"
        );
        SCALE
    };
    pub const ZERO: Self = Self::from_scaled_i64(0);
    pub const ONE: Self = Self::from_scaled_i64(Self::SCALE as i64);
    pub const QUANTUM: Self = Self::from_scaled_i64(1);
    pub const MIN: Self = Self::from_scaled_i64(i64::MIN);
    pub const MAX: Self = Self::from_scaled_i64(i64::MAX);

    /// Wrap a raw count of quanta; a raw `1` is one quantum.
    pub const fn from_scaled_i64(raw: i64) -> Self {
        let _ = Self::SCALE;
        Self(raw)
    }

    pub const fn to_scaled_i64(self) -> i64 {
        self.0
    }

    pub const fn from_le_bytes(bytes: [u8; 8]) -> Self {
        Self::from_scaled_i64(i64::from_le_bytes(bytes))
    }

    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }
}

impl<const SCALE: u64> FixedNumber for I64Scaled<SCALE> {
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;
    const QUANTUM: Self = Self::QUANTUM;

    fn checked_from_i128(value: i128) -> MathResult<Self> {
        value
            .checked_mul(i128::from(Self::SCALE))
            .and_then(|raw| i64::try_from(raw).ok())
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

    fn mul_div_floor(self, mul: Self, div: Self, _: Seal) -> MathResult<Floored<Self>> {
        if self.0 < 0 || mul.0 < 0 || div.0 < 0 {
            return Err(MathError::Domain);
        }
        let floor = unsigned_mul_div_floor(self.0 as u128, mul.0 as u128, div.0 as u128)?;
        Ok(Floored {
            value: Self(i64::try_from(floor.value).map_err(|_| MathError::Overflow)?),
            inexact: floor.inexact,
        })
    }

    fn ratio_floor(
        numerator: u128,
        denominator: NonZeroU128,
        _: Seal,
    ) -> MathResult<Floored<Self>> {
        let floor = unsigned_mul_div_floor(numerator, u128::from(Self::SCALE), denominator.get())?;
        Ok(Floored {
            value: Self(i64::try_from(floor.value).map_err(|_| MathError::Overflow)?),
            inexact: floor.inexact,
        })
    }

    fn checked_mul_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        self.checked_mul_div_round(rhs, Self::ONE, rounding)
    }

    fn checked_div_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        self.checked_mul_div_round(Self::ONE, rhs, rounding)
    }

    fn checked_mul_div_round(
        self,
        mul: Self,
        div: Self,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let floor = unsigned_mul_div_floor(
            u128::from(self.0.unsigned_abs()),
            u128::from(mul.0.unsigned_abs()),
            u128::from(div.0.unsigned_abs()),
        )?;
        signed_round(floor, (self.0 < 0) ^ (mul.0 < 0) ^ (div.0 < 0), rounding).map(Self)
    }

    fn checked_from_ratio_round(
        numerator: i128,
        denominator: NonZeroU128,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let floor = unsigned_mul_div_floor(
            numerator.unsigned_abs(),
            u128::from(Self::SCALE),
            denominator.get(),
        )?;
        signed_round(floor, numerator < 0, rounding).map(Self)
    }
}

impl<const SCALE: u64> SignedFixed for I64Scaled<SCALE> {
    fn checked_neg(self) -> MathResult<Self> {
        self.0.checked_neg().map(Self).ok_or(MathError::Overflow)
    }
}

impl<const SCALE: u64> IntegralRounding for I64Scaled<SCALE> {
    fn checked_floor_to_u64(self) -> MathResult<u64> {
        if self.0 < 0 {
            return Err(MathError::NegativeToUnsigned);
        }
        Ok(self.0 as u64 / Self::SCALE)
    }
}

fn signed_round(
    floor: Floored<u128>,
    negative: bool,
    rounding: DirectedRounding,
) -> MathResult<i64> {
    let increment = floor.inexact
        && matches!(
            (negative, rounding),
            (true, DirectedRounding::Floor) | (false, DirectedRounding::Ceil)
        );
    let magnitude = floor
        .value
        .checked_add(u128::from(increment))
        .ok_or(MathError::Overflow)?;
    if negative && magnitude == 1_u128 << 63 {
        return Ok(i64::MIN);
    }
    let raw = i64::try_from(magnitude).map_err(|_| MathError::Overflow)?;
    Ok(if negative { -raw } else { raw })
}

/// Unsigned 256-bit storage on the grid `1 / SCALE`, for any nonzero `u64` scale.
///
/// `U256Scaled<10>` has quantum `0.1`; `U256Scaled<1_000_000_000_000>` has
/// quantum `1e-12`. Both hold a full `u64 × u64` integral. Fused arithmetic
/// uses up to 512 bits before one final directed rounding. The implementation
/// is allocation-free and variable-time, for public accounting quantities.
/// Raw constructors, checked extraction, and bytes avoid requiring consumers
/// to depend on the underlying limb library.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U256Scaled<const SCALE: u64>(U256);

impl<const SCALE: u64> U256Scaled<SCALE> {
    pub const SCALE: u64 = {
        assert!(SCALE > 0, "scale must be positive");
        SCALE
    };
    pub const ZERO: Self = Self::from_scaled_u256(U256::ZERO);
    pub const ONE: Self = Self::from_scaled_u256(U256::from_u64(Self::SCALE));
    pub const QUANTUM: Self = Self::from_scaled_u256(U256::ONE);
    pub const MIN: Self = Self::ZERO;
    pub const MAX: Self = Self::from_scaled_u256(U256::MAX);

    pub const fn from_scaled_u128(raw: u128) -> Self {
        Self::from_scaled_u256(U256::from_u128(raw))
    }

    pub fn checked_to_scaled_u128(self) -> MathResult<u128> {
        if self.0.bits() > 128 {
            return Err(MathError::Overflow);
        }
        Ok(low_u128(self.0))
    }

    pub const fn from_scaled_u256(raw: U256) -> Self {
        let _ = Self::SCALE;
        Self(raw)
    }

    pub const fn to_scaled_u256(self) -> U256 {
        self.0
    }

    pub fn from_le_bytes(bytes: [u8; 32]) -> Self {
        Self::from_scaled_u256(U256::from_le_bytes(bytes))
    }

    pub fn to_le_bytes(self) -> [u8; 32] {
        self.0.to_le_bytes()
    }
}

impl<const SCALE: u64> UnsignedFixed for U256Scaled<SCALE> {}

impl<const SCALE: u64> From<u64> for U256Scaled<SCALE> {
    fn from(value: u64) -> Self {
        Self::from_scaled_u128(u128::from(value) * u128::from(Self::SCALE))
    }
}

impl<const SCALE: u64> FixedNumber for U256Scaled<SCALE> {
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;
    const QUANTUM: Self = Self::QUANTUM;

    fn checked_from_i128(value: i128) -> MathResult<Self> {
        let value = u128::try_from(value).map_err(|_| MathError::NegativeToUnsigned)?;
        // At most 127 + 64 bits, so the scaled integer always fits storage.
        Ok(Self(
            U256::from_u128(value).wrapping_mul(&U256::from_u64(Self::SCALE)),
        ))
    }

    fn checked_add(self, rhs: Self) -> MathResult<Self> {
        Option::<U256>::from(self.0.checked_add(&rhs.0))
            .map(Self)
            .ok_or(MathError::Overflow)
    }

    fn checked_sub(self, rhs: Self) -> MathResult<Self> {
        Option::<U256>::from(self.0.checked_sub(&rhs.0))
            .map(Self)
            .ok_or(MathError::Overflow)
    }

    fn mul_div_floor(self, mul: Self, div: Self, _: Seal) -> MathResult<Floored<Self>> {
        if div.0 == U256::ZERO {
            return Err(MathError::DivisionByZero);
        }
        if self.0.bits() + mul.0.bits() <= 128 && div.0.bits() <= 128 {
            let product = low_u128(self.0) * low_u128(mul.0);
            let divisor = low_u128(div.0);
            return Ok(Floored {
                value: Self::from_scaled_u128(product / divisor),
                inexact: !product.is_multiple_of(divisor),
            });
        }
        let divisor = NonZero::new(div.0.resize::<{ U512::LIMBS }>());
        let divisor = Option::<NonZero<U512>>::from(divisor).ok_or(MathError::DivisionByZero)?;
        let (low, high) = self.0.mul_wide(&mul.0);
        let (quotient, remainder) = high.concat(&low).div_rem(&divisor);
        if quotient.bits() > 256 {
            return Err(MathError::Overflow);
        }
        Ok(Floored {
            value: Self(quotient.resize()),
            inexact: remainder != U512::ZERO,
        })
    }

    fn ratio_floor(
        numerator: u128,
        denominator: NonZeroU128,
        _: Seal,
    ) -> MathResult<Floored<Self>> {
        if let Some(scaled) = numerator.checked_mul(u128::from(Self::SCALE)) {
            return Ok(Floored {
                value: Self::from_scaled_u128(scaled / denominator.get()),
                inexact: !scaled.is_multiple_of(denominator.get()),
            });
        }
        // A u128 numerator times a u64 scale fits within 192 bits.
        let scaled = U256::from_u128(numerator).wrapping_mul(&U256::from_u64(Self::SCALE));
        let divisor = NonZero::new(U256::from_u128(denominator.get()));
        let divisor = Option::<NonZero<U256>>::from(divisor).ok_or(MathError::DivisionByZero)?;
        let (quotient, remainder) = scaled.div_rem(&divisor);
        Ok(Floored {
            value: Self(quotient),
            inexact: remainder != U256::ZERO,
        })
    }
}

impl<const SCALE: u64> IntegralRounding for U256Scaled<SCALE> {
    fn checked_floor_to_u64(self) -> MathResult<u64> {
        let divisor = NonZero::new(U256::from_u64(Self::SCALE));
        let divisor = Option::<NonZero<U256>>::from(divisor).ok_or(MathError::DivisionByZero)?;
        let (whole, _) = self.0.div_rem(&divisor);
        if whole.bits() > 64 {
            return Err(MathError::Overflow);
        }
        Ok(low_u128(whole) as u64)
    }
}

fn low_u128(raw: U256) -> u128 {
    let bytes = raw.to_le_bytes();
    let mut low = [0_u8; 16];
    low.copy_from_slice(&bytes[..16]);
    u128::from_le_bytes(low)
}

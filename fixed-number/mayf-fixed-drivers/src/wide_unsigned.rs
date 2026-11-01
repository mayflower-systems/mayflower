//! Wide integral range on the existing 2^-64 quantum grid.
use crate::U64F64;
use crate::kernels::div_rem_wide;
use core::num::NonZeroU128;
use crypto_bigint::{CheckedAdd, CheckedSub, Encoding, U64, U256};
use mayf_fixed_number::{DirectedRounding, FixedNumber, IntegralRounding, MathError, MathResult};

/// Unsigned Q192.64 for cumulative deposit-time and other large integrals.
///
/// Its quantum matches `U64F64`, but its integer range accommodates a full
/// `u64` deposit multiplied by a full `u64` elapsed time. Arithmetic is checked
/// and uses one final directed rounding. Division is variable-time: this driver
/// is intended for public accounting quantities, not secret cryptographic data.
/// Integer products and divisors fitting 128 raw bits use short limb paths;
/// general wide divisors use bounded binary long division.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U192F64(U256);

impl mayf_fixed_number::UnsignedFixed for U192F64 {}

impl U192F64 {
    pub const FRAC_BITS: u32 = 64;
    pub const ZERO: Self = Self(U256::ZERO);
    pub const ONE: Self = Self(U256::ONE.shl_vartime(64));
    pub const QUANTUM: Self = Self(U256::ONE);
    pub const MIN: Self = Self::ZERO;
    pub const MAX: Self = Self(U256::MAX);

    pub const fn from_scaled_u256(value: U256) -> Self {
        Self(value)
    }
    pub const fn to_scaled_u256(self) -> U256 {
        self.0
    }
    pub const fn from_u64(value: u64) -> Self {
        Self(U256::from_u64(value).shl_vartime(64))
    }
    pub fn from_le_bytes(bytes: [u8; 32]) -> Self {
        Self(U256::from_le_bytes(bytes))
    }
    pub fn to_le_bytes(self) -> [u8; 32] {
        self.0.to_le_bytes()
    }
}

impl From<U64F64> for U192F64 {
    fn from(value: U64F64) -> Self {
        Self(U256::from_u128(value.to_scaled_u128()))
    }
}

impl TryFrom<U192F64> for U64F64 {
    type Error = MathError;

    fn try_from(value: U192F64) -> MathResult<Self> {
        if value.0.shr_vartime(128) != U256::ZERO {
            return Err(MathError::Overflow);
        }
        Ok(Self::from_scaled_u128(low_u128(value.0)))
    }
}

impl FixedNumber for U192F64 {
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;
    const QUANTUM: Self = Self::QUANTUM;

    fn mul_div_floor(
        self,
        mul: Self,
        div: Self,
        _: mayf_fixed_number::driver::Seal,
    ) -> MathResult<mayf_fixed_number::driver::Floored<Self>> {
        if div == Self::ZERO {
            return Err(MathError::DivisionByZero);
        }
        if mul == div {
            return Ok(mayf_fixed_number::driver::Floored {
                value: self,
                inexact: false,
            });
        }
        let (low, high) = self.0.mul_wide(&mul.0);
        let floor = divide_floor(high, low, div.0)?;
        Ok(mayf_fixed_number::driver::Floored {
            value: Self(floor.value),
            inexact: floor.inexact,
        })
    }

    fn ratio_floor(
        numerator: u128,
        denominator: NonZeroU128,
        _: mayf_fixed_number::driver::Seal,
    ) -> MathResult<mayf_fixed_number::driver::Floored<Self>> {
        let floor = divide_floor(
            U256::ZERO,
            U256::from_u128(numerator).shl_vartime(64),
            U256::from_u128(denominator.get()),
        )?;
        Ok(mayf_fixed_number::driver::Floored {
            value: Self(floor.value),
            inexact: floor.inexact,
        })
    }

    fn checked_from_i128(value: i128) -> MathResult<Self> {
        let value = u128::try_from(value).map_err(|_| MathError::NegativeToUnsigned)?;
        Ok(Self(U256::from_u128(value).shl_vartime(64)))
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

    fn checked_mul_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        // Deposits and elapsed ticks are integers. Their products need neither
        // a full 256x256 multiply nor a fixed-point truncation.
        if let Some(integer) = small_integer(rhs.0) {
            return mul_integer(self.0, integer).map(Self);
        }
        if let Some(integer) = small_integer(self.0) {
            return mul_integer(rhs.0, integer).map(Self);
        }
        let (low, high) = self.0.mul_wide(&rhs.0);
        if high.shr_vartime(64) != U256::ZERO {
            return Err(MathError::Overflow);
        }
        let quotient = low.shr_vartime(64).bitor(&high.shl_vartime(192));
        rounded(quotient, low_u128(low) as u64 != 0, rounding).map(Self)
    }
    fn checked_div_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        divide_round(
            self.0.shr_vartime(192),
            self.0.shl_vartime(64),
            rhs.0,
            rounding,
        )
        .map(Self)
    }
    fn checked_mul_div_round(
        self,
        multiplier: Self,
        divisor: Self,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        if divisor == Self::ZERO {
            return Err(MathError::DivisionByZero);
        }
        if multiplier == divisor {
            return Ok(self);
        }
        let (low, high) = self.0.mul_wide(&multiplier.0);
        divide_round(high, low, divisor.0, rounding).map(Self)
    }
    fn checked_from_ratio_round(
        numerator: i128,
        denominator: NonZeroU128,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let numerator = u128::try_from(numerator).map_err(|_| MathError::NegativeToUnsigned)?;
        divide_round(
            U256::ZERO,
            U256::from_u128(numerator).shl_vartime(64),
            U256::from_u128(denominator.get()),
            rounding,
        )
        .map(Self)
    }
}

impl IntegralRounding for U192F64 {
    fn checked_floor_to_u64(self) -> MathResult<u64> {
        if self.0.shr_vartime(128) != U256::ZERO {
            return Err(MathError::Overflow);
        }
        Ok((low_u128(self.0) >> 64) as u64)
    }
}

fn small_integer(raw: U256) -> Option<u64> {
    if raw.shr_vartime(128) == U256::ZERO {
        let raw = low_u128(raw);
        if raw as u64 == 0 {
            return Some((raw >> 64) as u64);
        }
    }
    None
}

fn mul_integer(raw: U256, integer: u64) -> MathResult<U256> {
    let (low, high) = raw.mul_wide(&U64::from_u64(integer));
    if high == U64::ZERO {
        Ok(low)
    } else {
        Err(MathError::Overflow)
    }
}

fn low_u128(value: U256) -> u128 {
    let bytes = value.to_le_bytes();
    let mut low = [0_u8; 16];
    low.copy_from_slice(&bytes[..16]);
    u128::from_le_bytes(low)
}

fn rounded(quotient: U256, inexact: bool, rounding: DirectedRounding) -> MathResult<U256> {
    if inexact && rounding == DirectedRounding::Ceil {
        Option::<U256>::from(quotient.checked_add(&U256::ONE)).ok_or(MathError::Overflow)
    } else {
        Ok(quotient)
    }
}

/// Divide the 512-bit pair by a 256-bit divisor. Reject a result wider than
/// storage before starting. The shared 256-by-128 limb divider handles the
/// common denominator (an integer child count shifted by 64) in two steps.
fn divide_round(
    high: U256,
    low: U256,
    divisor: U256,
    rounding: DirectedRounding,
) -> MathResult<U256> {
    let floor = divide_floor(high, low, divisor)?;
    rounded(floor.value, floor.inexact, rounding)
}

fn divide_floor(
    high: U256,
    low: U256,
    divisor: U256,
) -> MathResult<mayf_fixed_number::driver::Floored<U256>> {
    if divisor == U256::ZERO {
        return Err(MathError::DivisionByZero);
    }
    if high >= divisor {
        return Err(MathError::Overflow);
    }
    if divisor.shr_vartime(128) == U256::ZERO {
        let denominator = low_u128(divisor);
        let (q_high, remainder) =
            div_rem_wide(low_u128(high), low_u128(low.shr_vartime(128)), denominator);
        let (q_low, remainder) = div_rem_wide(remainder, low_u128(low), denominator);
        let quotient = U256::from_u128(q_high)
            .shl_vartime(128)
            .bitor(&U256::from_u128(q_low));
        return Ok(mayf_fixed_number::driver::Floored {
            value: quotient,
            inexact: remainder != 0,
        });
    }
    // Binary long division, bounded by the 256 possible quotient bits. Skip
    // leading zero numerator bits when the high half is empty. Each remainder
    // is below divisor; a carried shift is therefore reduced exactly once.
    let mut quotient = U256::ZERO;
    let mut remainder = high;
    let bits = if high == U256::ZERO {
        low.bits_vartime()
    } else {
        256
    };
    for bit in (0..bits).rev() {
        let carry = remainder.bit_vartime(255);
        remainder = remainder.shl_vartime(1);
        if low.bit_vartime(bit) {
            remainder = remainder.bitor(&U256::ONE);
        }
        if carry || remainder >= divisor {
            remainder = remainder.wrapping_sub(&divisor);
            quotient = quotient.bitor(&U256::ONE.shl_vartime(bit));
        }
    }
    Ok(mayf_fixed_number::driver::Floored {
        value: quotient,
        inexact: remainder != U256::ZERO,
    })
}

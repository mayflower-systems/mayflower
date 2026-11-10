//! Unsigned binary fixed-point drivers with matched integer/fraction widths.

use core::num::{NonZeroU64, NonZeroU128};

use crypto_bigint::{CheckedAdd, CheckedSub, Encoding, NonZero, U64, U128, U256};
use mayf_fixed_number::{DirectedRounding, FixedNumber, IntegralRounding, MathError, MathResult};

use crate::kernels::{div_rem_wide, wide_mul_u128};

const Q64_RAW_ONE: u128 = 1u128 << 64;
const Q64_FRACTION_MASK: u128 = Q64_RAW_ONE - 1;
const Q128_SHIFT: usize = 128;

/// Unsigned Q64.64 backed by a native `u128` (16 bytes).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U64F64(u128);

/// Unsigned Q128.128 fixed point backed by a `crypto_bigint::U256`.
///
/// This grid exists for one job: holding `1 / denominator` for a `u128`
/// denominator with enough precision that multiplying it back by a `u64`
/// numerator loses less than a quantum of the narrow grid. On [`U64F64`] that
/// reciprocal keeps only `2^64 / denominator` quanta, so a denominator near
/// `u64::MAX` would leave a share accurate to a few parts in one.
///
/// It deliberately does not implement [`FixedNumber`], and so has no
/// full-width multiplication, division, or multiply-divide. Those would need
/// general 512-bit intermediate arithmetic. This specialized type uses
/// variable-time operations for public accounting quantities instead:
/// addition, subtraction, ordering, multiplication by a [`U64F64`], an exact
/// raw ratio in `u64` terms, construction from a ratio, and narrowing back to
/// [`U64F64`]. Integer scaling uses quotient/remainder decomposition; mixed
/// multiplication uses a 384-bit product and a fixed shift, with no wide divider.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U128F128(U256);

impl mayf_fixed_number::UnsignedFixed for U64F64 {}

impl U64F64 {
    pub const FRAC_BITS: u32 = 64;
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(Q64_RAW_ONE);
    pub const QUANTUM: Self = Self(1);
    pub const MIN: Self = Self::ZERO;
    pub const MAX: Self = Self(u128::MAX);

    /// Wrap an already-scaled Q64.64 raw value.
    pub const fn from_scaled_u128(value: u128) -> Self {
        Self(value)
    }

    /// Return the Q64.64 raw value. A logical `1.0` is `2^64`.
    pub const fn to_scaled_u128(self) -> u128 {
        self.0
    }

    /// Construct an exact whole value.
    pub const fn from_u64(value: u64) -> Self {
        Self((value as u128) << Self::FRAC_BITS)
    }

    /// Restore a stable 16-byte little-endian representation.
    pub const fn from_le_bytes(bytes: [u8; 16]) -> Self {
        Self(u128::from_le_bytes(bytes))
    }

    /// Return a stable 16-byte little-endian persistence representation.
    pub const fn to_le_bytes(self) -> [u8; 16] {
        self.0.to_le_bytes()
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn checked_add(self, rhs: Self) -> MathResult<Self> {
        self.0
            .checked_add(rhs.0)
            .map(Self)
            .ok_or(MathError::Overflow)
    }

    pub fn checked_sub(self, rhs: Self) -> MathResult<Self> {
        self.0
            .checked_sub(rhs.0)
            .map(Self)
            .ok_or(MathError::Overflow)
    }

    /// Multiply the raw Q64.64 value by `multiplier`, divide by `divisor`, and
    /// round the single exact quotient in the requested direction.
    ///
    /// The fixed-point scale is unchanged. Decomposing `self` into quotient and
    /// remainder before multiplication avoids requiring a wider-than-`u128`
    /// intermediate while still computing exactly
    /// `round(self.raw * multiplier / divisor)`.
    pub fn checked_mul_div_u64_round(
        self,
        multiplier: u64,
        divisor: NonZeroU64,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let divisor_u128 = u128::from(divisor.get());
        let quotient = self.0 / divisor_u128;
        let remainder =
            u64::try_from(self.0 % divisor_u128).map_err(|_| MathError::UnsupportedRange)?;
        let quotient = quotient
            .checked_mul(u128::from(multiplier))
            .ok_or(MathError::Overflow)?;
        let (remainder_quotient, inexact) = mul_div_u64_remainder(remainder, multiplier, divisor)?;
        let quotient = quotient
            .checked_add(u128::from(remainder_quotient))
            .ok_or(MathError::Overflow)?;
        round_u128(quotient, inexact, rounding).map(Self)
    }

    /// Multiply two Q64.64 values exactly into the Q128.128 flavor.
    pub fn widening_mul(self, rhs: Self) -> U128F128 {
        let (high, low) = wide_mul_u128(self.0, rhs.0);
        let raw = U256::from_u128(low).bitor(&U256::from_u128(high).shl_vartime(128));
        U128F128::from_scaled_u256(raw)
    }
}

impl FixedNumber for U64F64 {
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;
    const QUANTUM: Self = Self::QUANTUM;

    fn mul_div_floor(
        self,
        mul: Self,
        div: Self,
        _: mayf_fixed_number::driver::Seal,
    ) -> MathResult<mayf_fixed_number::driver::Floored<Self>> {
        let floor = crate::kernels::unsigned_mul_div_floor(self.0, mul.0, div.0)?;
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
        let floor =
            crate::kernels::unsigned_mul_div_floor(numerator, Q64_RAW_ONE, denominator.get())?;
        Ok(mayf_fixed_number::driver::Floored {
            value: Self(floor.value),
            inexact: floor.inexact,
        })
    }

    fn checked_from_i128(value: i128) -> MathResult<Self> {
        let value = u128::try_from(value).map_err(|_| MathError::NegativeToUnsigned)?;
        value
            .checked_mul(Q64_RAW_ONE)
            .map(Self)
            .ok_or(MathError::Overflow)
    }

    fn checked_add(self, rhs: Self) -> MathResult<Self> {
        self.checked_add(rhs)
    }

    fn checked_sub(self, rhs: Self) -> MathResult<Self> {
        self.checked_sub(rhs)
    }

    fn checked_mul_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        let (high, low) = wide_mul_u128(self.0, rhs.0);
        if high >= Q64_RAW_ONE {
            return Err(MathError::Overflow);
        }

        let quotient = (high << 64) | (low >> 64);
        round_u128(quotient, low & Q64_FRACTION_MASK != 0, rounding).map(Self)
    }

    fn checked_div_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        if rhs.is_zero() {
            return Err(MathError::DivisionByZero);
        }
        let high = self.0 >> 64;
        let low = self.0 << 64;
        div_wide_u128_round(high, low, rhs.0, rounding).map(Self)
    }

    fn checked_mul_div_round(
        self,
        multiplier: Self,
        divisor: Self,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        if divisor.is_zero() {
            return Err(MathError::DivisionByZero);
        }
        let (high, low) = wide_mul_u128(self.0, multiplier.0);
        div_wide_u128_round(high, low, divisor.0, rounding).map(Self)
    }

    fn checked_from_ratio_round(
        numerator: i128,
        denominator: NonZeroU128,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let numerator = u128::try_from(numerator).map_err(|_| MathError::NegativeToUnsigned)?;
        let (high, low) = wide_mul_u128(numerator, Q64_RAW_ONE);
        div_wide_u128_round(high, low, denominator.get(), rounding).map(Self)
    }
}

impl IntegralRounding for U64F64 {
    fn checked_floor_to_u64(self) -> MathResult<u64> {
        Ok((self.0 >> Self::FRAC_BITS) as u64)
    }
}

impl U128F128 {
    pub const FRAC_BITS: u32 = 128;
    pub const ZERO: Self = Self(U256::ZERO);
    pub const ONE: Self = Self(U256::ONE.shl_vartime(Q128_SHIFT));
    pub const QUANTUM: Self = Self(U256::ONE);
    pub const MIN: Self = Self::ZERO;
    pub const MAX: Self = Self(U256::MAX);

    /// Wrap an already-scaled Q128.128 raw value.
    pub const fn from_scaled_u256(value: U256) -> Self {
        Self(value)
    }

    /// Return the Q128.128 raw value. A logical `1.0` is `2^128`.
    pub const fn to_scaled_u256(self) -> U256 {
        self.0
    }

    /// Construct an exact whole value.
    pub fn from_u64(value: u64) -> Self {
        Self(U256::from_u64(value).shl_vartime(Q128_SHIFT))
    }

    /// Restore a stable 32-byte little-endian representation.
    pub const fn from_le_bytes(bytes: [u8; 32]) -> Self {
        Self(U256::from_le_slice(&bytes))
    }

    /// Return a stable 32-byte little-endian persistence representation.
    pub fn to_le_bytes(self) -> [u8; 32] {
        self.0.to_le_bytes()
    }

    pub fn is_zero(self) -> bool {
        self.0 == U256::ZERO
    }

    pub fn checked_add(self, rhs: Self) -> MathResult<Self> {
        Option::<U256>::from(self.0.checked_add(&rhs.0))
            .map(Self)
            .ok_or(MathError::Overflow)
    }

    pub fn checked_sub(self, rhs: Self) -> MathResult<Self> {
        Option::<U256>::from(self.0.checked_sub(&rhs.0))
            .map(Self)
            .ok_or(MathError::Overflow)
    }

    /// Multiply the raw Q128.128 value by `multiplier`, divide by `divisor`,
    /// and round the single exact quotient in the requested direction.
    ///
    /// This is the exact integer operation
    /// `round(self.raw * multiplier / divisor)`; it does not rescale the fixed
    /// value. The quotient/remainder decomposition keeps every large
    /// intermediate in `U256` (with only a `U64` overflow limb) instead of
    /// constructing the `U512` product used by fixed-by-fixed arithmetic.
    pub fn checked_mul_div_u64_round(
        self,
        multiplier: u64,
        divisor: NonZeroU64,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let (quotient, remainder) = div_rem_u256_u64(self.0, divisor);
        let quotient = checked_mul_u256_u64(quotient, multiplier)?;
        let (remainder_quotient, inexact) = mul_div_u64_remainder(remainder, multiplier, divisor)?;
        let quotient =
            Option::<U256>::from(quotient.checked_add(&U256::from_u64(remainder_quotient)))
                .ok_or(MathError::Overflow)?;
        round_u256(quotient, inexact, rounding).map(Self)
    }

    /// Multiply Q128.128 by Q64.64 without promoting both operands to U512.
    ///
    /// The exact raw result is `round(self.raw * rhs.raw / 2^64)`. A mixed
    /// U256-by-U128 product produces only 384 bits; shifting that product by
    /// the known binary scale needs no general division.
    pub fn checked_mul_u64f64_round(
        self,
        rhs: U64F64,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let (low, high) = self.0.mul_wide(&U128::from_u128(rhs.0));
        if high.shr_vartime(64) != U128::ZERO {
            return Err(MathError::Overflow);
        }

        let quotient = low
            .shr_vartime(64)
            .bitor(&U256::from(&high).shl_vartime(192));
        let low_bytes = low.to_le_bytes();
        let inexact = low_bytes[..8].iter().any(|byte| *byte != 0);
        round_u256(quotient, inexact, rounding).map(Self)
    }

    pub fn checked_mul_u64f64_floor(self, rhs: U64F64) -> MathResult<Self> {
        self.checked_mul_u64f64_round(rhs, DirectedRounding::Floor)
    }

    pub fn checked_mul_u64f64_ceil(self, rhs: U64F64) -> MathResult<Self> {
        self.checked_mul_u64f64_round(rhs, DirectedRounding::Ceil)
    }

    /// Narrow onto the Q64.64 grid with an explicit rounding direction.
    pub fn checked_to_u64f64_round(self, rounding: DirectedRounding) -> MathResult<U64F64> {
        let raw_bytes = self.0.to_le_bytes();
        let shifted = self.0.shr_vartime(64).to_le_bytes();
        if shifted[16..].iter().any(|byte| *byte != 0) {
            return Err(MathError::Overflow);
        }

        let mut quotient_bytes = [0u8; 16];
        quotient_bytes.copy_from_slice(&shifted[..16]);
        let quotient = u128::from_le_bytes(quotient_bytes);
        let inexact = raw_bytes[..8].iter().any(|byte| *byte != 0);
        round_u128(quotient, inexact, rounding).map(U64F64::from_scaled_u128)
    }

    pub fn checked_to_u64f64_floor(self) -> MathResult<U64F64> {
        self.checked_to_u64f64_round(DirectedRounding::Floor)
    }

    pub fn checked_to_u64f64_ceil(self) -> MathResult<U64F64> {
        self.checked_to_u64f64_round(DirectedRounding::Ceil)
    }

    /// Build `numerator / denominator` on this grid, rounded as asked.
    pub fn checked_from_ratio_round(
        numerator: i128,
        denominator: NonZeroU128,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let numerator = u128::try_from(numerator).map_err(|_| MathError::NegativeToUnsigned)?;
        let denominator = denominator.get();

        // The exact result is `numerator * 2^128 / denominator`. Splitting it
        // at the fixed-point scale keeps both halves native: the whole part is
        // a `u128` division, and its remainder is below the divisor, which is
        // exactly the precondition `div_rem_wide` needs for the fraction.
        //
        // `crypto_bigint::U256::div_rem` computes the same value, but it is
        // constant-time and cannot exit early. This variable-time path is
        // intended for public accounting quantities, not secrets.
        let whole = numerator / denominator;
        let (fraction, remainder) = div_rem_wide(numerator % denominator, 0, denominator);

        let quotient = U256::from_u128(whole)
            .shl_vartime(Q128_SHIFT)
            .bitor(&U256::from_u128(fraction));
        round_u256(quotient, remainder != 0, rounding).map(Self)
    }
}
impl From<U64F64> for U128F128 {
    fn from(value: U64F64) -> Self {
        Self(U256::from_u128(value.0).shl_vartime(64))
    }
}

fn round_u128(quotient: u128, inexact: bool, rounding: DirectedRounding) -> MathResult<u128> {
    if inexact && rounding == DirectedRounding::Ceil {
        quotient.checked_add(1).ok_or(MathError::Overflow)
    } else {
        Ok(quotient)
    }
}

fn round_u256(quotient: U256, inexact: bool, rounding: DirectedRounding) -> MathResult<U256> {
    if inexact && rounding == DirectedRounding::Ceil {
        Option::<U256>::from(quotient.checked_add(&U256::ONE)).ok_or(MathError::Overflow)
    } else {
        Ok(quotient)
    }
}

/// Divide a U256 by a non-zero u64 without widening the dividend.
fn div_rem_u256_u64(value: U256, divisor: NonZeroU64) -> (U256, u64) {
    #[cfg(target_pointer_width = "64")]
    {
        let (quotient, remainder) = value.div_rem_limb(NonZero::from(divisor));
        (quotient, remainder.0)
    }

    #[cfg(target_pointer_width = "32")]
    {
        let (quotient, remainder) = value.div_rem(&NonZero::<U256>::from(divisor));
        let bytes = remainder.to_le_bytes();
        let mut low = [0u8; 8];
        low.copy_from_slice(&bytes[..8]);
        (quotient, u64::from_le_bytes(low))
    }
}

/// Multiply a U256 by a u64 and reject a non-zero overflow limb.
fn checked_mul_u256_u64(value: U256, multiplier: u64) -> MathResult<U256> {
    let (low, high) = value.mul_wide(&U64::from_u64(multiplier));
    if high == U64::ZERO {
        Ok(low)
    } else {
        Err(MathError::Overflow)
    }
}

/// Exact quotient and inexactness for `(remainder * multiplier) / divisor`.
/// Both factors are u64, so their product fits in u128.
fn mul_div_u64_remainder(
    remainder: u64,
    multiplier: u64,
    divisor: NonZeroU64,
) -> MathResult<(u64, bool)> {
    let product = u128::from(remainder) * u128::from(multiplier);
    let divisor = u128::from(divisor.get());
    let quotient = u64::try_from(product / divisor).map_err(|_| MathError::Overflow)?;
    Ok((quotient, product % divisor != 0))
}

fn div_wide_u128_round(
    high: u128,
    low: u128,
    divisor: u128,
    rounding: DirectedRounding,
) -> MathResult<u128> {
    if divisor == 0 {
        return Err(MathError::DivisionByZero);
    }
    if high >= divisor {
        return Err(MathError::Overflow);
    }

    let (quotient, remainder) = div_rem_wide(high, low, divisor);
    round_u128(quotient, remainder != 0, rounding)
}

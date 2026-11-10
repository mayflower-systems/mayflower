//! Signed Q64.64 arithmetic and certified exponential enclosures.

use crate::kernels::{muldiv_raw, signed_from_magnitude, unsigned_mul_div_floor};
use core::{fmt, num::NonZeroU128};
use fixed::types::I64F64 as FixedI64F64;
use mayf_fixed_number::{
    BoundedExponential, Bounds, DirectedRounding, FixedNumber, IntegralRounding, MathError,
    MathResult, SignedFixed,
};

const RAW_ONE: i128 = 1i128 << 64;

/// One-quantum enclosure of `ln 2` on the `2^-64` grid.
///
/// `ln 2 * 2^64 = 12786308645202655659.7886...`, so the lower edge is the
/// exact floor. The oracle test suite re-derives both edges from a
/// high-precision `ln 2` and fails if either constant is wrong.
const LN2_LO_RAW: i128 = 12_786_308_645_202_655_659;
const LN2_HI_RAW: i128 = LN2_LO_RAW + 1;

/// `expm1(x)` exceeds `i128` raw range just above `x = 63*ln2 ~ 43.67`, so
/// inputs at or above `44` are rejected before range reduction. Between the
/// true edge and this guard the checked reconstruction shift reports the
/// overflow itself.
const EXPM1_POSITIVE_OVERFLOW_GUARD_RAW: i128 = 44 * RAW_ONE;

/// For `x <= -45`, `exp(x) <= e^-45 ~ 2.87e-20`, which is below one `2^-64`
/// quantum (`~5.42e-20`), so `[-1, -1 + quantum]` is a certified enclosure of
/// `expm1(x)` without any computation.
const EXPM1_NEGATIVE_QUANTUM_CUTOFF_RAW: i128 = -45 * RAW_ONE;

/// Taylor terms evaluated for `expm1(r)` on the reduced interval. For
/// `r <= 0.7` the tail after 24 terms is bounded by
/// `0.7^25 / 25! * (1 / (1 - 0.7/26)) < 9e-30`, far below one quantum.
const EXPM1_SERIES_TERMS: i128 = 24;

/// Upper-edge allowance covering the truncated series tail. One quantum
/// (`2^-64 ~ 5.42e-20`) exceeds the `9e-30` tail bound by ten orders of
/// magnitude.
const EXPM1_SERIES_TAIL_ALLOWANCE_RAW: i128 = 1;

/// Reduced arguments live in `[0, ln2)` up to the interval slack introduced
/// by the one-quantum `ln 2` enclosure, which is at most `|k| <= 65` quanta.
/// Arguments outside `[-SLACK, ln2 + SLACK]` indicate a reduction bug and are
/// reported as [`MathError::Domain`] instead of being silently bounded.
const EXPM1_REDUCTION_SLACK_RAW: i128 = 1 << 10;

/// Signed fixed-point number on the binary `2^-64` grid.
///
/// The wrapped value is a [`fixed::types::I64F64`]: `1.0` is stored as
/// `2^64`, and one quantum is stored as `1`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct I64F64(FixedI64F64);

impl I64F64 {
    /// Number of fractional bits in the binary grid.
    pub const FRAC_BITS: u32 = 64;

    pub const ZERO: Self = Self(FixedI64F64::ZERO);
    pub const ONE: Self = Self(FixedI64F64::ONE);
    pub const QUANTUM: Self = Self(FixedI64F64::DELTA);

    /// Largest raw value representable by this driver.
    pub const MAX: Self = Self(FixedI64F64::MAX);

    /// Smallest raw value representable by this driver.
    pub const MIN: Self = Self(FixedI64F64::MIN);

    /// Wrap an already-scaled `2^-64` grid value.
    pub const fn from_scaled_i128(value: i128) -> Self {
        Self(FixedI64F64::from_bits(value))
    }

    /// Return the raw scaled integer. A logical `1.0` is `2^64`.
    pub const fn to_scaled_i128(self) -> i128 {
        self.0.to_bits()
    }

    /// Restore the raw signed Q64.64 bits from stable little-endian storage.
    pub const fn from_le_bytes(bytes: [u8; 16]) -> Self {
        Self::from_scaled_i128(i128::from_le_bytes(bytes))
    }

    /// Serialize the raw signed Q64.64 bits without changing precision.
    pub const fn to_le_bytes(self) -> [u8; 16] {
        self.to_scaled_i128().to_le_bytes()
    }
}

impl fmt::Display for I64F64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<i32> for I64F64 {
    fn from(value: i32) -> Self {
        Self::checked_from_i128(i128::from(value)).expect("i32 always fits in I64F64")
    }
}

impl FixedNumber for I64F64 {
    const ZERO: Self = Self(FixedI64F64::ZERO);
    const ONE: Self = Self(FixedI64F64::ONE);
    const QUANTUM: Self = Self(FixedI64F64::DELTA);

    fn mul_div_floor(
        self,
        mul: Self,
        div: Self,
        _: mayf_fixed_number::driver::Seal,
    ) -> MathResult<mayf_fixed_number::driver::Floored<Self>> {
        if self < Self::ZERO || mul < Self::ZERO || div < Self::ZERO {
            return Err(MathError::Domain);
        }
        let floor = unsigned_mul_div_floor(
            self.to_scaled_i128() as u128,
            mul.to_scaled_i128() as u128,
            div.to_scaled_i128() as u128,
        )?;
        let raw = i128::try_from(floor.value).map_err(|_| MathError::Overflow)?;
        Ok(mayf_fixed_number::driver::Floored {
            value: Self::from_scaled_i128(raw),
            inexact: floor.inexact,
        })
    }

    fn ratio_floor(
        numerator: u128,
        denominator: NonZeroU128,
        _: mayf_fixed_number::driver::Seal,
    ) -> MathResult<mayf_fixed_number::driver::Floored<Self>> {
        let floor = unsigned_mul_div_floor(numerator, RAW_ONE as u128, denominator.get())?;
        let raw = i128::try_from(floor.value).map_err(|_| MathError::Overflow)?;
        Ok(mayf_fixed_number::driver::Floored {
            value: Self::from_scaled_i128(raw),
            inexact: floor.inexact,
        })
    }

    fn checked_from_i128(value: i128) -> MathResult<Self> {
        value
            .checked_mul(RAW_ONE)
            .map(Self::from_scaled_i128)
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
        muldiv_raw(
            self.to_scaled_i128(),
            rhs.to_scaled_i128(),
            RAW_ONE,
            rounding,
        )
        .map(Self::from_scaled_i128)
    }

    fn checked_div_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        muldiv_raw(
            self.to_scaled_i128(),
            RAW_ONE,
            rhs.to_scaled_i128(),
            rounding,
        )
        .map(Self::from_scaled_i128)
    }

    fn checked_mul_div_round(
        self,
        multiplier: Self,
        divisor: Self,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        muldiv_raw(
            self.to_scaled_i128(),
            multiplier.to_scaled_i128(),
            divisor.to_scaled_i128(),
            rounding,
        )
        .map(Self::from_scaled_i128)
    }

    fn checked_from_ratio_round(
        numerator: i128,
        denominator: NonZeroU128,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        // Keep the denominator unsigned: its upper half is a valid input even
        // though it cannot be represented by an i128. Round the magnitude away
        // from zero for negative floors and positive ceilings only.
        let floor =
            unsigned_mul_div_floor(numerator.unsigned_abs(), RAW_ONE as u128, denominator.get())?;
        let negative = numerator < 0;
        let away = match rounding {
            DirectedRounding::Floor => negative,
            DirectedRounding::Ceil => !negative,
        };
        let magnitude = if floor.inexact && away {
            floor.value.checked_add(1).ok_or(MathError::Overflow)?
        } else {
            floor.value
        };
        signed_from_magnitude(magnitude, negative).map(Self::from_scaled_i128)
    }
}

impl SignedFixed for I64F64 {
    fn checked_neg(self) -> MathResult<Self> {
        self.0.checked_neg().map(Self).ok_or(MathError::Overflow)
    }
}

impl IntegralRounding for I64F64 {
    fn checked_floor_to_u64(self) -> MathResult<u64> {
        let raw = self.to_scaled_i128();
        if raw < 0 {
            return Err(MathError::NegativeToUnsigned);
        }
        Ok((raw / RAW_ONE) as u64)
    }
}

impl BoundedExponential for I64F64 {
    fn checked_exp_m1_bounds(self) -> MathResult<Bounds<Self>> {
        let x = self.to_scaled_i128();
        if x == 0 {
            return Ok(Bounds::exact(Self::ZERO));
        }
        if x >= EXPM1_POSITIVE_OVERFLOW_GUARD_RAW {
            return Err(MathError::Overflow);
        }
        if x <= EXPM1_NEGATIVE_QUANTUM_CUTOFF_RAW {
            return Bounds::new(
                Self::from_scaled_i128(-RAW_ONE),
                Self::from_scaled_i128(-RAW_ONE + 1),
            );
        }

        // Exact range reduction: `x = k*ln2 + r` with `r` in `[0, ln2)` up to
        // the one-quantum `ln 2` interval. `|x| < 45` bounds `|k| <= 65`, so
        // `k * LN2_*_RAW` stays far inside `i128` and every step is exact.
        let k = x.div_euclid(LN2_LO_RAW);
        if !(-128..=127).contains(&k) {
            return Err(MathError::Domain);
        }
        let scaled_ln2_lo = k.checked_mul(LN2_LO_RAW).ok_or(MathError::Overflow)?;
        let scaled_ln2_hi = k.checked_mul(LN2_HI_RAW).ok_or(MathError::Overflow)?;
        let (r_lo, r_hi) = if k >= 0 {
            (
                x.checked_sub(scaled_ln2_hi).ok_or(MathError::Overflow)?,
                x.checked_sub(scaled_ln2_lo).ok_or(MathError::Overflow)?,
            )
        } else {
            (
                x.checked_sub(scaled_ln2_lo).ok_or(MathError::Overflow)?,
                x.checked_sub(scaled_ln2_hi).ok_or(MathError::Overflow)?,
            )
        };

        // `expm1` is increasing, so per-endpoint directed evaluation encloses
        // every remainder in `[r_lo, r_hi]`.
        let series_lo = expm1_reduced_raw(r_lo, DirectedRounding::Floor)?;
        let series_hi = expm1_reduced_raw(r_hi, DirectedRounding::Ceil)?;

        let lower = reconstruct_raw(series_lo, k, DirectedRounding::Floor)?;
        let upper = reconstruct_raw(series_hi, k, DirectedRounding::Ceil)?;
        Bounds::new(Self::from_scaled_i128(lower), Self::from_scaled_i128(upper))
    }
}

/// Directed bound of `expm1(r)` for a reduced raw argument.
///
/// Preconditions: `r` lies in `[-SLACK, ln2 + SLACK]`; violations are a
/// reduction bug and return [`MathError::Domain`].
///
/// Slightly negative arguments only arise from the `ln 2` interval slack and
/// are bounded linearly: `t <= expm1(t) <= t + t^2/2` for `t <= 0`, and
/// `t^2/2` is below one quantum for `|t| <= SLACK`.
///
/// Non-negative arguments use the positive-term Taylor sum. With floor
/// rounding every computed term is at most its true value, so the sum is a
/// lower bound of the true partial sum and therefore of `expm1(r)`. With ceil
/// rounding the sum dominates the true partial sum, and the one-quantum tail
/// allowance covers the truncated remainder.
fn expm1_reduced_raw(r: i128, rounding: DirectedRounding) -> MathResult<i128> {
    if r < -EXPM1_REDUCTION_SLACK_RAW {
        return Err(MathError::Domain);
    }
    if r > LN2_HI_RAW + EXPM1_REDUCTION_SLACK_RAW {
        return Err(MathError::Domain);
    }
    if r < 0 {
        return match rounding {
            DirectedRounding::Floor => Ok(r),
            DirectedRounding::Ceil => Ok(r + 1),
        };
    }

    let mut term = r;
    let mut sum = r;
    let mut n = 2i128;
    while n <= EXPM1_SERIES_TERMS {
        let divisor = n.checked_mul(RAW_ONE).ok_or(MathError::Overflow)?;
        term = muldiv_raw(term, r, divisor, rounding)?;
        sum = sum.checked_add(term).ok_or(MathError::Overflow)?;
        n += 1;
    }

    match rounding {
        DirectedRounding::Floor => Ok(sum),
        DirectedRounding::Ceil => sum
            .checked_add(EXPM1_SERIES_TAIL_ALLOWANCE_RAW)
            .ok_or(MathError::Overflow),
    }
}

/// Reconstruct `2^k * (1 + s) - 1` in raw form with one directed rounding.
///
/// `1 + s` and the final subtraction are exact grid arithmetic. For `k >= 0`
/// the shift is exact and checked for overflow; for `k < 0` the right shift
/// is the single rounding step, floor via arithmetic shift and ceil via
/// pre-biasing. The map is increasing in `s`, so feeding the lower series
/// bound with floor rounding and the upper series bound with ceil rounding
/// preserves the enclosure.
fn reconstruct_raw(s: i128, k: i128, rounding: DirectedRounding) -> MathResult<i128> {
    let base = RAW_ONE.checked_add(s).ok_or(MathError::Overflow)?;
    if base <= 0 {
        return Err(MathError::Domain);
    }

    let scaled = if k >= 0 {
        let shift = u32::try_from(k).map_err(|_| MathError::Domain)?;
        if shift >= 127 || base > (i128::MAX >> shift) {
            return Err(MathError::Overflow);
        }
        base << shift
    } else {
        let shift = u32::try_from(-k).map_err(|_| MathError::Domain)?;
        if shift >= 127 {
            return Err(MathError::Domain);
        }
        match rounding {
            DirectedRounding::Floor => base >> shift,
            DirectedRounding::Ceil => {
                let bias = (1i128 << shift) - 1;
                base.checked_add(bias).ok_or(MathError::Overflow)? >> shift
            }
        }
    };

    scaled.checked_sub(RAW_ONE).ok_or(MathError::Overflow)
}

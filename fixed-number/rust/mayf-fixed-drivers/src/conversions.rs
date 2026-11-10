//! Lossless widening and checked, exact conversion between public formats.
//!
//! TryFrom never silently discards fraction bits. Quantizing conversions have
//! explicit rounding in their names instead.

use mayf_fixed_number::{MathError, MathResult};

use crate::{I64F64, U64F64, U128F128, U192F64};

impl From<i64> for I64F64 {
    fn from(value: i64) -> Self {
        Self::from_scaled_i128(i128::from(value) << 64)
    }
}

impl From<u64> for U64F64 {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl From<u64> for U192F64 {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl From<u64> for U128F128 {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl TryFrom<U64F64> for I64F64 {
    type Error = MathError;

    fn try_from(value: U64F64) -> MathResult<Self> {
        i128::try_from(value.to_scaled_u128())
            .map(Self::from_scaled_i128)
            .map_err(|_| MathError::Overflow)
    }
}

impl TryFrom<I64F64> for U64F64 {
    type Error = MathError;

    fn try_from(value: I64F64) -> MathResult<Self> {
        u128::try_from(value.to_scaled_i128())
            .map(Self::from_scaled_u128)
            .map_err(|_| MathError::NegativeToUnsigned)
    }
}

impl TryFrom<I64F64> for U192F64 {
    type Error = MathError;

    fn try_from(value: I64F64) -> MathResult<Self> {
        U64F64::try_from(value).map(Self::from)
    }
}

impl TryFrom<I64F64> for U128F128 {
    type Error = MathError;

    fn try_from(value: I64F64) -> MathResult<Self> {
        U64F64::try_from(value).map(Self::from)
    }
}

impl TryFrom<U128F128> for U64F64 {
    type Error = MathError;

    fn try_from(value: U128F128) -> MathResult<Self> {
        let result = value.checked_to_u64f64_floor()?;
        if U128F128::from(result) != value {
            return Err(MathError::Inexact);
        }
        Ok(result)
    }
}

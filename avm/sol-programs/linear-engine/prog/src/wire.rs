//! Borsh representation of normalized unsigned Q64.64 quantities.
//! A raw value of `1 << 64` represents one; no token normalization occurs here.

use anchor_lang::{prelude::*, solana_program::program::MAX_RETURN_DATA};
use mayf_avm_calculator::{SpanRequest as Request, SpanResponse as Response};
use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::{Bounds, MathError, NonNeg};

/// Each response contains six Borsh `u128`s; the returned vector has a `u32` length.
pub const MAX_BATCH_SPANS: usize = (MAX_RETURN_DATA - 4) / (6 * 16);

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanRequest {
    pub lo_q64x64: u128,
    /// Zero requests a point evaluation with zero area.
    pub width_q64x64: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Q64x64Bounds {
    pub lower: u128,
    pub upper: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanResponse {
    pub y_at_lo: Q64x64Bounds,
    pub y_at_hi: Q64x64Bounds,
    pub area: Q64x64Bounds,
}

impl From<SpanRequest> for Request<U64F64> {
    fn from(value: SpanRequest) -> Self {
        Self {
            lo: NonNeg::from_unsigned(U64F64::from_scaled_u128(value.lo_q64x64)),
            width: NonNeg::from_unsigned(U64F64::from_scaled_u128(value.width_q64x64)),
        }
    }
}

impl From<Request<U64F64>> for SpanRequest {
    fn from(value: Request<U64F64>) -> Self {
        Self {
            lo_q64x64: value.lo.get().to_scaled_u128(),
            width_q64x64: value.width.get().to_scaled_u128(),
        }
    }
}

impl From<Bounds<NonNeg<U64F64>>> for Q64x64Bounds {
    fn from(value: Bounds<NonNeg<U64F64>>) -> Self {
        Self {
            lower: value.lower().get().to_scaled_u128(),
            upper: value.upper().get().to_scaled_u128(),
        }
    }
}

impl TryFrom<Q64x64Bounds> for Bounds<NonNeg<U64F64>> {
    type Error = MathError;

    fn try_from(value: Q64x64Bounds) -> core::result::Result<Self, Self::Error> {
        Self::new(
            NonNeg::from_unsigned(U64F64::from_scaled_u128(value.lower)),
            NonNeg::from_unsigned(U64F64::from_scaled_u128(value.upper)),
        )
    }
}

impl From<Response<U64F64>> for SpanResponse {
    fn from(value: Response<U64F64>) -> Self {
        Self {
            y_at_lo: value.y_at_lo.into(),
            y_at_hi: value.y_at_hi.into(),
            area: value.area.into(),
        }
    }
}

impl TryFrom<SpanResponse> for Response<U64F64> {
    type Error = MathError;

    fn try_from(value: SpanResponse) -> core::result::Result<Self, Self::Error> {
        Ok(Self {
            y_at_lo: value.y_at_lo.try_into()?,
            y_at_hi: value.y_at_hi.try_into()?,
            area: value.area.try_into()?,
        })
    }
}

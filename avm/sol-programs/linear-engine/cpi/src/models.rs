//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]
use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)] // Some programs' models use only primitive fields.
use solana_address::Address;

// types -------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct CalculateSpansArgs {
    pub spans: Vec<SpanRequest>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreateLinearEngineArgs {
    pub slope_q64x64: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Q64x64Bounds {
    pub lower: u128,
    pub upper: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanRequest {
    pub lo_q64x64: u128,
    /// Zero requests a point evaluation with zero area.
    pub width_q64x64: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanResponse {
    pub y_at_lo: Q64x64Bounds,
    pub y_at_hi: Q64x64Bounds,
    pub area: Q64x64Bounds,
}

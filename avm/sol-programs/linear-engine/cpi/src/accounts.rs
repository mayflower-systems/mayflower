//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]
use borsh::{BorshDeserialize, BorshSerialize};
use solana_address::Address;

use crate::AccountData;
#[allow(unused_imports)] // Some programs' accounts use only primitive fields.
use crate::models::*;

// types -------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinearCharacteristicParam {
    /// Positive, immutable slope of `f(x) = slope * x`, on the `2^-64` grid.
    pub slope_q64x64: u128,
}

// account discriminators --------------------------------------------

impl AccountData for LinearCharacteristicParam {
    const DISCRIMINATOR: [u8; 8] = [168, 158, 201, 134, 101, 136, 37, 119];
}

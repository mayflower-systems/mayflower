//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]
use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)] // Some programs' models use only primitive fields.
use solana_address::Address;

// types -------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreateEmitterArgs {
    pub emission_rate_q64f64: u128,
    pub annual_decay_rate_q64f64: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetEmitterDecayRateArgs {
    pub annual_decay_rate_q64f64: u128,
}

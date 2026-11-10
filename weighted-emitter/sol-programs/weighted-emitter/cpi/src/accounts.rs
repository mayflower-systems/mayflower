//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]
use borsh::{BorshDeserialize, BorshSerialize};
use solana_address::Address;

use crate::AccountData;
#[allow(unused_imports)] // Some programs' accounts use only primitive fields.
use crate::models::*;

// types -------------------------------------------------------------

/// Canonical schedule and aggregate allocation state for one independent emitter.
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Emitter {
    pub authority: Address,
    pub vote_mint: Address,
    pub emission_mint: Address,
    pub mint_authority_bump: [u8; 1],
    pub emission_rate_q64f64: u128,
    pub annual_decay_rate_q64f64: u128,
    pub last_updated_at_unix_seconds: u64,
    /// Recorded deposits across gauges, independent of unsolicited escrow transfers.
    pub total_deposited: u64,
    /// Whole emission allocated to gauges, including amounts not yet minted.
    pub total_emitted: u64,
    pub total_minted: u64,
    pub emission_index_q192f64: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gauge {
    pub emitter: Address,
    pub total_deposited: u64,
    /// Accrued whole emission, including allocations checkpointed by vote changes.
    pub total_emitted: u64,
    /// The authority may mint the difference between total_emitted and total_minted.
    pub total_minted: u64,
    pub last_updated_at_unix_seconds: u64,
    pub emission_index_seen_q192f64: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VotePosition {
    pub authority: Address,
    pub gauge: Address,
    pub vote_escrow: Address,
    /// Recorded principal; external token donations do not add voting weight.
    pub deposited: u64,
    pub self_bump: [u8; 1],
}

// account discriminators --------------------------------------------

impl AccountData for Emitter {
    const DISCRIMINATOR: [u8; 8] = [4, 60, 118, 104, 34, 153, 156, 218];
}

impl AccountData for Gauge {
    const DISCRIMINATOR: [u8; 8] = [9, 19, 249, 189, 158, 171, 226, 205];
}

impl AccountData for VotePosition {
    const DISCRIMINATOR: [u8; 8] = [121, 54, 167, 23, 147, 96, 61, 209];
}

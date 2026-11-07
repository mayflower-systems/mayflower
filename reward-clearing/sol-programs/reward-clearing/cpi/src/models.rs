//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]
use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)] // Some programs' models use only primitive fields.
use solana_address::Address;

// types -------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelCreateArgs {
    pub default_epoch_terms: EpochTerms,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelSetDefaultEpochTermsArgs {
    pub default_epoch_terms: EpochTerms,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelSetDisabledArgs {
    pub disabled: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreateTenantArgs {
    pub master_start_time_unix_seconds: u64,
    pub epoch_duration_seconds: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochTerms {
    /// Ramp bonus slope in micro-basis points
    pub ramp_bonus_slope_mbps: u32,
    /// Exit penalty exponent
    pub exit_penalty_exponent: u8,
}

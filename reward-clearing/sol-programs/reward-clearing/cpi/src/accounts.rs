//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]
use borsh::{BorshDeserialize, BorshSerialize};
use solana_address::Address;

use crate::AccountData;
#[allow(unused_imports)] // Some programs' accounts use only primitive fields.
use crate::models::*;

// types -------------------------------------------------------------

/// A mint-specific reward channel synchronized by its tenant's UTC schedule.
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Channel {
    pub tenant: Address,
    pub reward_token_mint: Address,
    /// Channel-owned vault backing every `UserChannel::staged_reward_balance`.
    /// Epoch pots never fund another epoch through this account.
    pub reward_staging_token_account: Address,
    pub default_epoch_terms: EpochTerms,
    /// Disabling a channel prevents new epochs; existing epochs remain usable.
    pub disabled: bool,
    /// Creation-time bump retained for channel-owned token transfers.
    pub self_bump: [u8; 1],
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelEpoch {
    /// Link to channel
    pub channel: Address,
    /// Link to epoch account that contains all the channels
    pub epoch: Address,
    /// Token account that holds the reward pot for this epoch for this channel
    pub reward_pot_escrow_token_account: Address,
    /// Token account that holds the pledged clearing credits for this epoch for this channel
    pub clearing_credits_escrow_token_account: Address,
    /// Recorded balance of the clearing credits for this epoch for this channel
    pub clearing_credits_balance: u64,
    /// Recorded balance of the reward pot for this epoch for this channel
    pub reward_pot_balance: u64,
    /// Q64.64 fixed point number of weighted shares in this epoch for this channel
    pub weighted_shares_balance: u128,
    /// Terms for this epoch
    pub terms: EpochTerms,
    pub self_bump: [u8; 1],
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelEpochFinal {
    /// Link to channel epoch that is completed
    pub channel_epoch: Address,
    /// Mutable raw Q64.64 shares that have not yet claimed. This is the closure
    /// semaphore: claims decrement it by their exact position shares, and only
    /// zero permits the finalized account and pot to close.
    pub unclaimed_weighted_shares: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Epoch {
    pub tenant: Address,
    pub epoch_counter: u32,
    pub start_time_unix_seconds: u64,
    pub duration_seconds: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Organization {
    pub authority: Address,
    pub next_tenant_index: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tenant {
    pub organization: Address,
    pub tenant_index: u32,
    pub clearing_credits_mint: Address,
    pub master_start_time_unix_seconds: u64,
    pub epoch_duration_seconds: u32,
    /// Monotonic epoch index for this tenant.
    pub current_epoch_index: u32,
    /// Creation-time bump used only to sign the permissionless burn.
    pub self_bump: [u8; 1],
}

// account discriminators --------------------------------------------

impl AccountData for Channel {
    const DISCRIMINATOR: [u8; 8] = [49, 159, 99, 106, 220, 87, 219, 88];
}

impl AccountData for ChannelEpoch {
    const DISCRIMINATOR: [u8; 8] = [42, 194, 102, 118, 28, 161, 115, 40];
}

impl AccountData for ChannelEpochFinal {
    const DISCRIMINATOR: [u8; 8] = [37, 105, 18, 75, 194, 232, 198, 104];
}

impl AccountData for Epoch {
    const DISCRIMINATOR: [u8; 8] = [93, 83, 120, 89, 151, 138, 152, 108];
}

impl AccountData for Organization {
    const DISCRIMINATOR: [u8; 8] = [145, 38, 152, 251, 91, 57, 118, 160];
}

impl AccountData for Tenant {
    const DISCRIMINATOR: [u8; 8] = [61, 43, 215, 51, 232, 242, 209, 170];
}

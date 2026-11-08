//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]
use borsh::{BorshDeserialize, BorshSerialize};
use solana_address::Address;

use crate::AccountData;
#[allow(unused_imports)] // Some programs' accounts use only primitive fields.
use crate::models::*;

// types -------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct BidderPosition {
    pub authority: Address,
    pub channel: Address,
    pub epoch_counter: u32,
    pub claimed: bool,
    pub self_bump: [u8; 1],
    /// Oldest to newest. Allocated space is retained when withdrawals remove lots.
    pub lots: Vec<BidderLot>,
}

/// A mint-specific reward channel synchronized by its channel group's UTC schedule.
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Channel {
    pub channel_group: Address,
    pub reward_token_mint: Address,
    /// Channel-owned reward staging vault, initialized when the channel is created.
    pub reward_staging_token_account: Address,
    pub default_epoch_terms: EpochTerms,
    /// The most recent epoch opened by the state machine, starting at channel creation.
    pub last_created_epoch_counter: u32,
    /// Disabling a channel prevents new epochs; existing epochs remain usable.
    pub disabled: bool,
    /// Creation-time bump retained for channel-owned token transfers.
    pub self_bump: [u8; 1],
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelEpoch {
    /// Link to channel
    pub channel: Address,
    /// Epoch index within the channel's group.
    pub epoch_counter: u32,
    /// Token account that holds the reward pot for this epoch for this channel
    pub reward_pot_escrow_token_account: Address,
    /// Token account that holds the pledged clearing credits for this epoch for this channel
    pub clearing_credits_escrow_token_account: Address,
    /// Recorded balance of the clearing credits for this epoch for this channel
    pub clearing_credits_balance: u64,
    /// Recorded balance of the reward pot for this epoch for this channel
    pub reward_pot_balance: u64,
    /// Q64.64 fixed point number of weighted shares in this epoch for this channel
    pub weighted_shares_balance_q64f64: u128,
    /// Terms for this epoch
    pub terms: StoredEpochTerms,
    /// Original pot and total shares, frozen at finalization for every claim.
    pub final_totals: Option<StoredFinalizedTotals>,
    pub self_bump: [u8; 1],
}

/// A numbered channel group with a fixed schedule and a mutable epoch index.
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelGroup {
    pub organization: Address,
    pub channel_group_index: u32,
    pub clearing_credits_mint: Address,
    pub master_start_time_unix_seconds: u64,
    pub epoch_duration_seconds: u32,
    /// Monotonic epoch index for this channel group.
    pub current_epoch_index: u32,
    /// Creation-time bump used only to sign the permissionless burn.
    pub self_bump: [u8; 1],
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Epoch {
    pub channel_group: Address,
    pub epoch_counter: u32,
    pub start_time_unix_seconds: u64,
    pub duration_seconds: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Organization {
    pub authority: Address,
    pub next_channel_group_index: u32,
}

// account discriminators --------------------------------------------

impl AccountData for BidderPosition {
    const DISCRIMINATOR: [u8; 8] = [86, 37, 176, 141, 39, 7, 120, 255];
}

impl AccountData for Channel {
    const DISCRIMINATOR: [u8; 8] = [49, 159, 99, 106, 220, 87, 219, 88];
}

impl AccountData for ChannelEpoch {
    const DISCRIMINATOR: [u8; 8] = [42, 194, 102, 118, 28, 161, 115, 40];
}

impl AccountData for ChannelGroup {
    const DISCRIMINATOR: [u8; 8] = [36, 177, 239, 118, 135, 238, 130, 4];
}

impl AccountData for Epoch {
    const DISCRIMINATOR: [u8; 8] = [93, 83, 120, 89, 151, 138, 152, 108];
}

impl AccountData for Organization {
    const DISCRIMINATOR: [u8; 8] = [145, 38, 152, 251, 91, 57, 118, 160];
}

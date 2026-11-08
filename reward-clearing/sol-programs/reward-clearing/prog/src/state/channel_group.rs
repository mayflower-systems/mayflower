use anchor_lang::prelude::*;
use mayf_reward_clearing::models::channel_group::ChannelGroup as LibChannelGroup;

use crate::{constants::CHANNEL_GROUP_SEED, state::com::AccountId};

/// A numbered channel group with a fixed schedule and a mutable epoch index.
#[account]
#[derive(InitSpace)]
pub struct ChannelGroup {
    pub organization: Pubkey,
    pub channel_group_index: u32,
    pub clearing_credits_mint: Pubkey,
    pub master_start_time_unix_seconds: u64,
    pub epoch_duration_seconds: u32,

    /// Monotonic epoch index for this channel group.
    pub current_epoch_index: u32,

    /// Creation-time bump used only to sign the permissionless burn.
    pub self_bump: [u8; 1],
}

impl ChannelGroup {
    pub fn to_library(&self, id: Pubkey) -> LibChannelGroup<AccountId> {
        LibChannelGroup {
            id: AccountId(id),
            master_start_time_unix_seconds: self.master_start_time_unix_seconds,
            epoch_duration_seconds: self.epoch_duration_seconds.into(),
            current_epoch_index: self.current_epoch_index,
        }
    }

    pub fn signer_seeds<'a>(&'a self, index: &'a [u8; 4]) -> [&'a [u8]; 4] {
        [
            CHANNEL_GROUP_SEED,
            self.organization.as_ref(),
            index,
            self.self_bump.as_ref(),
        ]
    }
}

use anchor_lang::prelude::*;
use mayf_fixed_drivers::U64F64;
use mayf_reward_clearing::models::channel::Channel as LibChannel;

use crate::{
    constants::REWARD_CHANNEL_SEED,
    state::com::{AccountId, EpochTerms},
};

/// A mint-specific reward channel synchronized by its channel group's UTC schedule.
#[account]
#[derive(InitSpace)]
pub struct Channel {
    pub channel_group: Pubkey,

    pub reward_token_mint: Pubkey,

    /// Channel-owned reward staging vault, initialized when the channel is created.
    pub reward_staging_token_account: Pubkey,

    // Defaults snapshotted by newly created epochs.
    pub default_epoch_terms: EpochTerms,

    /// The most recent epoch opened by the state machine, starting at channel creation.
    pub last_created_epoch_counter: u32,

    /// Disabling a channel prevents new epochs; existing epochs remain usable.
    pub disabled: bool,

    /// Creation-time bump retained for channel-owned token transfers.
    pub self_bump: [u8; 1],
}

impl Channel {
    pub fn to_library(
        &self,
        id: Pubkey,
    ) -> mayf_reward_clearing::Result<LibChannel<AccountId, U64F64>> {
        Ok(LibChannel {
            id: AccountId(id),
            channel_group_id: AccountId(self.channel_group),
            reward_id: AccountId(self.reward_token_mint),
            default_terms: self.default_epoch_terms.try_into()?,
            disabled: self.disabled,
            last_created_epoch_counter: self.last_created_epoch_counter,
        })
    }

    pub fn signer_seeds(&self) -> [&[u8]; 4] {
        [
            REWARD_CHANNEL_SEED,
            self.channel_group.as_ref(),
            self.reward_token_mint.as_ref(),
            self.self_bump.as_ref(),
        ]
    }
}

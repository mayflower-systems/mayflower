use anchor_lang::prelude::*;

use crate::{constants::REWARD_CHANNEL_SEED, state::com::EpochTerms};

/// A mint-specific reward channel synchronized by its tenant's UTC schedule.
#[account]
#[derive(InitSpace)]
pub struct Channel {
    pub tenant: Pubkey,

    pub reward_token_mint: Pubkey,

    /// Channel-owned vault backing every `UserChannel::staged_reward_balance`.
    /// Epoch pots never fund another epoch through this account.
    pub reward_staging_token_account: Pubkey,

    // Defaults snapshotted by newly created epochs.
    pub default_epoch_terms: EpochTerms,

    /// Disabling a channel prevents new epochs; existing epochs remain usable.
    pub disabled: bool,

    /// Creation-time bump retained for channel-owned token transfers.
    pub self_bump: [u8; 1],
}

impl Channel {
    pub fn signer_seeds(&self) -> [&[u8]; 4] {
        [
            REWARD_CHANNEL_SEED,
            self.tenant.as_ref(),
            self.reward_token_mint.as_ref(),
            self.self_bump.as_ref(),
        ]
    }
}

use anchor_lang::prelude::*;

use crate::{constants::CHANNEL_EPOCH_SEED, state::com::EpochTerms};

#[account]
#[derive(InitSpace)]
pub struct ChannelEpoch {
    /// Link to channel
    pub channel: Pubkey,

    /// Link to epoch account that contains all the channels
    pub epoch: Pubkey,

    /// Token account that holds the reward pot for this epoch for this channel
    pub reward_pot_escrow_token_account: Pubkey,

    /// Token account that holds the pledged clearing credits for this epoch for this channel
    pub clearing_credits_escrow_token_account: Pubkey,

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

impl ChannelEpoch {
    pub fn signer_seeds(&self) -> [&[u8]; 4] {
        [
            CHANNEL_EPOCH_SEED,
            self.channel.as_ref(),
            self.epoch.as_ref(),
            self.self_bump.as_ref(),
        ]
    }
}

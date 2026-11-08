use anchor_lang::prelude::*;

use crate::{
    constants::CHANNEL_EPOCH_SEED,
    state::com::{AccountId, StoredEpochTerms},
};
use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::NonNeg;
use mayf_reward_clearing::RewardClearingError;
use mayf_reward_clearing::models::channel_epoch::{
    ChannelEpoch as LibChannelEpoch, FinalizedTotals as LibFinalizedTotals,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct StoredFinalizedTotals {
    pub reward_pot: u64,
    pub weighted_shares_q64f64: u128,
}

#[account]
#[derive(InitSpace)]
pub struct ChannelEpoch {
    /// Link to channel
    pub channel: Pubkey,

    /// Epoch index within the channel's group.
    pub epoch_counter: u32,

    /// Token account that holds the reward pot for this epoch for this channel
    pub reward_pot_escrow_token_account: Pubkey,

    /// Token account that holds the pledged clearing credits for this epoch for this channel
    pub clearing_credits_escrow_token_account: Pubkey,

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

impl ChannelEpoch {
    pub fn from_library(
        state: LibChannelEpoch<AccountId, U64F64>,
        reward_pot_escrow_token_account: Pubkey,
        clearing_credits_escrow_token_account: Pubkey,
        self_bump: u8,
    ) -> Self {
        Self {
            channel: state.channel_id.0,
            epoch_counter: state.epoch_counter,
            reward_pot_escrow_token_account,
            clearing_credits_escrow_token_account,
            clearing_credits_balance: state.clearing_credits_balance,
            reward_pot_balance: state.reward_pot_balance,
            weighted_shares_balance_q64f64: state.weighted_shares_balance.get().to_scaled_u128(),
            terms: state.terms.into(),
            final_totals: state.final_totals.map(|totals| StoredFinalizedTotals {
                reward_pot: totals.reward_pot,
                weighted_shares_q64f64: totals.weighted_shares.get().to_scaled_u128(),
            }),
            self_bump: [self_bump],
        }
    }

    pub fn signer_seeds<'a>(&'a self, epoch_counter: &'a [u8; 4]) -> [&'a [u8]; 4] {
        [
            CHANNEL_EPOCH_SEED,
            self.channel.as_ref(),
            epoch_counter,
            self.self_bump.as_ref(),
        ]
    }
}

impl TryFrom<&ChannelEpoch> for LibChannelEpoch<AccountId, U64F64> {
    type Error = RewardClearingError;

    fn try_from(state: &ChannelEpoch) -> mayf_reward_clearing::Result<Self> {
        Ok(Self {
            channel_id: AccountId(state.channel),
            epoch_counter: state.epoch_counter,
            reward_pot_balance: state.reward_pot_balance,
            clearing_credits_balance: state.clearing_credits_balance,
            weighted_shares_balance: NonNeg::from_unsigned(U64F64::from_scaled_u128(
                state.weighted_shares_balance_q64f64,
            )),
            terms: state.terms.try_into()?,
            final_totals: state.final_totals.map(|totals| LibFinalizedTotals {
                reward_pot: totals.reward_pot,
                weighted_shares: NonNeg::from_unsigned(U64F64::from_scaled_u128(
                    totals.weighted_shares_q64f64,
                )),
            }),
        })
    }
}

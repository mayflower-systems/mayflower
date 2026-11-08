use anchor_lang::prelude::*;
use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::NonNeg;
use mayf_reward_clearing::models::{
    bidder_position::{BidderLot as LibBidderLot, BidderPosition as LibBidderPosition},
    channel_epoch::ChannelEpochId,
};

use crate::state::com::AccountId;

#[account]
pub struct BidderPosition {
    pub authority: Pubkey,
    pub channel: Pubkey,
    pub epoch_counter: u32,
    pub claimed: bool,
    pub self_bump: [u8; 1],
    /// Oldest to newest. Allocated space is retained when withdrawals remove lots.
    pub lots: Vec<BidderLot>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct BidderLot {
    pub credits: u64,
    pub shares_q64f64: u128,
}

impl BidderPosition {
    /// Discriminator, fixed fields, and the Borsh Vec length prefix.
    pub const BASE_SPACE: usize = 8 + 32 + 32 + 4 + 1 + 1 + 4;

    pub fn space(&self) -> usize {
        Self::BASE_SPACE + self.lots.len() * BidderLot::INIT_SPACE
    }

    pub fn to_library(&self, id: Pubkey) -> LibBidderPosition<AccountId, U64F64> {
        let id = AccountId(id);
        LibBidderPosition {
            id,
            channel_epoch_id: ChannelEpochId {
                channel_id: AccountId(self.channel),
                epoch_counter: self.epoch_counter,
            },
            claimed: self.claimed,
            lots: self
                .lots
                .iter()
                .map(|lot| LibBidderLot {
                    bidder_position_id: id,
                    credits: lot.credits,
                    shares: NonNeg::from_unsigned(U64F64::from_scaled_u128(lot.shares_q64f64)),
                })
                .collect(),
        }
    }

    pub fn from_library(
        state: LibBidderPosition<AccountId, U64F64>,
        authority: Pubkey,
        self_bump: u8,
    ) -> Self {
        Self {
            authority,
            channel: state.channel_epoch_id.channel_id.0,
            epoch_counter: state.channel_epoch_id.epoch_counter,
            claimed: state.claimed,
            self_bump: [self_bump],
            lots: state
                .lots
                .into_iter()
                .map(|lot| BidderLot {
                    credits: lot.credits,
                    shares_q64f64: lot.shares.get().to_scaled_u128(),
                })
                .collect(),
        }
    }
}

//! The actions, one per module. The crate documentation explains how a host applies them.

mod add_reward;
mod advance_epoch;
mod claim;
mod create_bidder_position;
mod create_channel;
mod create_channel_epoch;
mod create_channel_group;
mod deposit_credits;
mod finalize_channel_epoch;
mod withdraw_credits;

pub use add_reward::{AddRewardArgs, AddRewardReceipt, add_reward};
pub use advance_epoch::{AdvanceEpochArgs, AdvanceEpochReceipt, advance_epoch};
pub use claim::{ClaimArgs, ClaimReceipt, claim};
pub use create_bidder_position::{
    CreateBidderPositionArgs, CreateBidderPositionReceipt, create_bidder_position,
};
pub use create_channel::{CreateChannelArgs, CreateChannelReceipt, create_channel};
pub use create_channel_epoch::{
    CreateChannelEpochArgs, CreateChannelEpochReceipt, create_channel_epoch,
};
pub use create_channel_group::{
    CreateChannelGroupArgs, CreateChannelGroupReceipt, MIN_EPOCH_DURATION_SECONDS,
    create_channel_group,
};
pub use deposit_credits::{DepositCreditsArgs, DepositCreditsReceipt, deposit_credits};
pub use finalize_channel_epoch::{
    FinalizeChannelEpochArgs, FinalizeChannelEpochReceipt, finalize_channel_epoch,
};
pub use withdraw_credits::{WithdrawCreditsArgs, WithdrawCreditsReceipt, withdraw_credits};

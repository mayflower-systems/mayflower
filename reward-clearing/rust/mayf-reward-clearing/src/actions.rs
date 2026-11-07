//! The actions, one per module. The crate documentation explains how a host applies them.

mod add_reward;
mod advance_epoch;
mod claim;
mod deposit_credits;
mod finalize_channel_epoch;
mod withdraw_credits;

pub use add_reward::{AddRewardArgs, AddRewardReceipt, add_reward};
pub use advance_epoch::{AdvanceEpochArgs, AdvanceEpochReceipt, advance_epoch};
pub use claim::{ClaimArgs, ClaimReceipt, claim};
pub use deposit_credits::{DepositCreditsArgs, DepositCreditsReceipt, deposit_credits};
pub use finalize_channel_epoch::{
    FinalizeChannelEpochArgs, FinalizeChannelEpochReceipt, finalize_channel_epoch,
};
pub use withdraw_credits::{WithdrawCreditsArgs, WithdrawCreditsReceipt, withdraw_credits};

//! Public transitions. Each action consumes drafts of the mutable records and
//! returns the next records plus explicit custody effects. Store the entire
//! receipt and perform its transfers atomically, or store nothing on error.
//!
//! Membership: auction activation or a permanent parent join. Balances: one
//! asset plus its ancestors, bounded by the configured maximum depth. Family
//! terms: every source lane of one parent, up to the sixteen-lane storage
//! capacity, including shared upstream residuals. Collection
//! mints only already-accrued net credits; it does not implicitly load sources.

mod collect_credits;
mod create_protocol;
mod join_at_top_level;
mod join_under_parent;
mod register_lane;
mod set_balance;
mod set_credit_stake;
mod set_offer;
mod set_sharing;
mod settle_family;
mod settle_node_in_lane;

pub use collect_credits::{CollectCreditsArgs, CollectCreditsReceipt, collect_credits};
pub use create_protocol::{CreateProtocolArgs, CreateProtocolReceipt, create_protocol};
pub use join_at_top_level::{JoinAtTopLevelArgs, JoinAtTopLevelReceipt, join_at_top_level};
pub use join_under_parent::{JoinUnderParentArgs, JoinUnderParentReceipt, join_under_parent};
pub use register_lane::{RegisterLaneArgs, RegisterLaneReceipt, register_lane};
pub use set_balance::{SetBalanceArgs, SetBalanceReceipt, set_balance};
pub use set_credit_stake::{SetCreditStakeArgs, SetCreditStakeReceipt, set_credit_stake};
pub use set_offer::{SetOfferArgs, SetOfferReceipt, set_offer};
pub use set_sharing::{SetSharingArgs, SetSharingReceipt, set_sharing};
pub use settle_family::{SettleFamilyArgs, SettleFamilyReceipt, settle_family};
pub use settle_node_in_lane::{SettleNodeInLaneArgs, SettleNodeInLaneReceipt, settle_node_in_lane};

#[cfg(test)]
mod tests;

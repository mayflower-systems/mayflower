//! The program: every action on the kernel, one per module.
//!
//! An action is one state transition. It takes an `Args` struct of records and inputs, by
//! value, and returns a `Receipt`. The receipt holds each record the action changed, named
//! `*_next`, each record it created, named `new_*`, and each amount the host must move or
//! mint. The host stores every record in the receipt and drops the rest of the args. An
//! action that fails returns an [`Error`](crate::Error) and no records, so the host stores
//! nothing.
//!
//! # The network
//!
//! - [`create_protocol()`]: start a network, with its terms, its emission rate, and the price of
//!   a top-level seat
//! - [`set_emission_rate()`]: change the rate of emission
//!
//! # Lanes
//!
//! - [`register_lane()`]: add a lane for one deposit token
//! - [`set_gauge_vote()`]: set one voter's votes on one lane
//!
//! # Membership
//!
//! - [`join_at_top_level()`]: admit a new member to a top-level seat, for the top-level fee
//! - [`join_under_parent()`]: admit a new member directly under a parent, for the parent's fee
//! - [`set_offer()`]: change a node's price for new children
//! - [`set_top_level_offer()`]: change the price of a top-level seat
//!
//! # A member's credits
//!
//! - [`set_sharing()`]: change the portion of its credits a member shares with its children
//! - [`collect_credits()`]: take the credits a member keeps, from every lane
//!
//! # A member's lanes
//!
//! - [`deposit_into_lane()`]: add to a member's balance in a lane
//! - [`withdraw_from_lane()`]: remove from a member's balance in a lane
//! - [`settle_node_in_lane()`]: bring the credits a node and its lineage earned in a lane into
//!   their nodes. Anyone may call it.

mod collect_credits;
mod create_protocol;
mod deposit_into_lane;
mod join_at_top_level;
mod join_under_parent;
mod register_lane;
mod set_emission_rate;
mod set_gauge_vote;
mod set_offer;
mod set_sharing;
mod set_top_level_offer;
mod settle_node_in_lane;
mod withdraw_from_lane;

pub use collect_credits::{CollectCreditsArgs, CollectCreditsReceipt, collect_credits};
pub use create_protocol::{CreateProtocolArgs, CreateProtocolReceipt, create_protocol};
pub use deposit_into_lane::{DepositIntoLaneArgs, DepositIntoLaneReceipt, deposit_into_lane};
pub use join_at_top_level::{JoinAtTopLevelArgs, JoinAtTopLevelReceipt, join_at_top_level};
pub use join_under_parent::{JoinUnderParentArgs, JoinUnderParentReceipt, join_under_parent};
pub use register_lane::{RegisterLaneArgs, RegisterLaneReceipt, register_lane};
pub use set_emission_rate::{SetEmissionRateArgs, SetEmissionRateReceipt, set_emission_rate};
pub use set_gauge_vote::{SetGaugeVoteArgs, SetGaugeVoteReceipt, set_gauge_vote};
pub use set_offer::{SetOfferArgs, SetOfferReceipt, set_offer};
pub use set_sharing::{SetSharingArgs, SetSharingReceipt, set_sharing};
pub use set_top_level_offer::{SetTopLevelOfferArgs, SetTopLevelOfferReceipt, set_top_level_offer};
pub use settle_node_in_lane::{SettleNodeInLaneArgs, SettleNodeInLaneReceipt, settle_node_in_lane};
pub use withdraw_from_lane::{WithdrawFromLaneArgs, WithdrawFromLaneReceipt, withdraw_from_lane};

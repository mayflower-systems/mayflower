#![forbid(unsafe_code)]

//! A state machine for distributing a funded reward pot through a timed auction.
//!
//! Bidders commit credits for weighted shares of the pot. The [entry weight]
//! sets the shares created by a deposit. The [exit penalty] sets how much
//! withdrawn principal is forfeited. After the deadline, finalization commits
//! the remaining credits to be burned, and surviving shares determine payouts.
//!
//! [entry weight]: models::channel_epoch::ChannelEpoch::entry_weight
//! [exit penalty]: models::channel_epoch::ChannelEpoch::exit_penalty
//!
//! # Lifecycle
//!
//! - [`create_channel_group`] sets the shared schedule and creates epoch zero.
//!   [`create_channel`] opens a channel and its first auction in the group's
//!   current live epoch.
//! - During that epoch, fund the pot with [`add_reward`] and open positions with
//!   [`create_bidder_position`]. Use [`deposit_credits`] and [`withdraw_credits`]
//!   to change commitments.
//! - After the deadline, [`finalize_channel_epoch`] freezes the pot and shares.
//!   Each position with surviving shares can then [`claim`] once.
//! - [`advance_epoch`] moves the group forward one epoch at a time.
//!   [`create_channel_epoch`] opens later auctions for an existing channel.
//!   Earlier auctions can finalize and pay claims independently.
//!
//! # Host integration
//!
//! Use the action functions for state transitions. Each takes an `Args` struct
//! and checks the supplied models, lifecycle, and arithmetic. A successful
//! action returns a `Receipt` with the new or updated models and any computed
//! settlement amounts.
//!
//! The host chooses the identifier and number types, loads the authoritative
//! state, authenticates callers, and supplies trusted execution time. It
//! enforces unique identities on creation and applies the receipt against the
//! same state used to compute it. The state update and associated escrow
//! transfers, refunds, burns, and payouts must succeed atomically.
//!
//! # Numbers
//!
//! Credits and rewards are whole `u64` units ([`Amount`]). [`Shares`], weights,
//! and fractions are [`NonNeg<N>`](mayf_fixed_number::NonNeg) for the host's
//! fixed-point type `N`. Choose a range that fits the expected balances and
//! weighted shares. Entry weights, newly issued shares, refunds, and payouts
//! round down. Exit penalties and partial-lot share removals round up.

mod actions;
mod auction;
mod error;
pub mod models;
#[cfg(test)]
mod test_support;

pub use actions::*;
pub use error::{Result, RewardClearingError};
pub use models::epoch_terms::EpochTerms;
pub use models::{Amount, EpochCounter, Seconds, Shares};

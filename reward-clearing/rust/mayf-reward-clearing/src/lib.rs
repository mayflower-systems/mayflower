#![forbid(unsafe_code)]

//! The reward-clearing auction as pure functions.
//!
//! Bidders deposit credits into a channel epoch and receive weighted shares of its
//! reward pot. Earlier deposits earn more shares per credit. A withdrawal forfeits
//! a penalty that grows through the epoch. After the epoch ends, the host burns the
//! remaining credits, and each position claims its part of the pot. The wiki's
//! Revenue Clearing pages specify the algorithm.
//!
//! # Actions and the host
//!
//! Each action takes an `Args` struct of models and inputs. It returns either an
//! error or a `Receipt` with the models it changed or created, and any amounts the
//! host must move. The host writes every model in the receipt and nothing else.
//!
//! The models are normalized, like rows in database tables linked by ID. Each
//! action validates everything except authorization: that its models belong
//! together, the timing, and the amounts.
//!
//! The host loads the models, applies each receipt atomically, and moves the tokens.
//! It owns authorization, storage, custody, and funding fees.
//!
//! # Numbers
//!
//! Credits and rewards are whole `u64` units ([`Amount`]). [`Shares`], weights,
//! and fractions are [`NonNeg<N>`](mayf_fixed_number::NonNeg) for the host's
//! fixed-number type `N`. Every rounding step favors the protocol. Entry weights,
//! shares, returned credits, and rewards round down. Exit penalties and the shares
//! a withdrawal removes round up.

mod actions;
mod error;
pub mod models;
#[cfg(test)]
mod test_support;

pub use actions::*;
pub use error::{Result, RewardClearingError};
pub use models::epoch_terms::EpochTerms;
pub use models::{Amount, EpochCounter, Seconds, Shares};

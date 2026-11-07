//! The state that actions read and return.
//! The host stores these models and chooses their identifier type `I`.

pub mod bidder_position;
pub mod channel;
pub mod channel_epoch;
pub mod channel_epoch_final;
pub mod channel_group;
pub mod epoch;
pub mod epoch_terms;

use mayf_fixed_number::NonNeg;

/// A whole number of credits or reward units. Fractional units are never stored.
pub type Amount = u64;

/// A timestamp or duration in seconds
pub type Seconds = u64;

/// The index of an epoch in its channel group's schedule
pub type EpochCounter = u32;

/// Weighted shares of a channel epoch's reward pot
pub type Shares<N> = NonNeg<N>;

/// An identifier chosen by the host, such as an account address
pub trait Identifiable: PartialEq + Clone + Copy {}

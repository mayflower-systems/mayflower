//! The state that actions read and return.
//! The host stores these models and chooses their identifier type `I`.

pub mod bidder_position;
pub mod channel;
pub mod channel_epoch;
pub mod channel_epoch_final;
pub mod channel_group;
pub mod epoch;
pub mod epoch_terms;

/// An identifier chosen by the host, such as an account address
pub trait Identifiable: PartialEq + Clone + Copy {}

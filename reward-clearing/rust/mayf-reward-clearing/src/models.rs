pub mod bidder_position;
pub mod channel;
pub mod channel_epoch;
pub mod channel_group;
pub mod epoch;

pub trait Identifiable: PartialEq + Clone + Copy {}

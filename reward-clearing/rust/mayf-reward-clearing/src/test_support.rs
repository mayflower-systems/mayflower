//! Fixtures shared by the action unit tests.

use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::models::Identifiable;
use crate::models::bidder_position::BidderPosition;
use crate::models::channel::Channel;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::epoch::Epoch;
use crate::{
    Amount, DepositCreditsArgs, DepositCreditsReceipt, EpochTerms, Result, Seconds, Shares,
};

pub type Number = U64F64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Id(pub u8);
impl Identifiable for Id {}

pub const CHANNEL_GROUP: Id = Id(0);
pub const CHANNEL: Id = Id(1);
pub const BIDDER: Id = Id(2);
pub const START: Seconds = 1_000;
pub const DURATION: Seconds = 100;
pub const HALFWAY: Seconds = START + DURATION / 2;

pub fn epoch() -> Epoch<Id> {
    Epoch::new(CHANNEL_GROUP, 3, START, DURATION)
}

pub fn terms(ramp_bonus_slope: i32, exit_penalty_exponent: u32) -> EpochTerms<Number> {
    let slope = Number::from_i32(ramp_bonus_slope).unwrap();
    EpochTerms::new(slope, exit_penalty_exponent).unwrap()
}

/// A channel in the epoch's channel group, with the wiki's reference terms
pub fn channel() -> Channel<Id, Number> {
    Channel {
        id: CHANNEL,
        channel_group_id: CHANNEL_GROUP,
        reward_id: Id(5),
        default_terms: terms(1, 2),
        last_seen_epoch_counter: None,
    }
}

/// The wiki's reference terms: ramp bonus slope 1 and exit penalty exponent 2.
/// Entry weight is 2 at the start and 1.5 halfway. The exit penalty is 25% halfway.
pub fn channel_epoch() -> ChannelEpoch<Id, Number> {
    channel_epoch_with_terms(1, 2)
}

pub fn channel_epoch_with_terms(
    ramp_bonus_slope: i32,
    exit_penalty_exponent: u32,
) -> ChannelEpoch<Id, Number> {
    ChannelEpoch::new(CHANNEL, 3, terms(ramp_bonus_slope, exit_penalty_exponent))
}

pub fn empty_position(channel_epoch: &ChannelEpoch<Id, Number>) -> BidderPosition<Id, Number> {
    BidderPosition {
        id: BIDDER,
        channel_epoch_id: channel_epoch.id(),
        lots: Vec::new(),
        claimed: false,
    }
}

pub fn deposit(
    bidder_position: BidderPosition<Id, Number>,
    channel_epoch: ChannelEpoch<Id, Number>,
    now: Seconds,
    credits_to_deposit: Amount,
) -> Result<DepositCreditsReceipt<Id, Number>> {
    crate::deposit_credits(DepositCreditsArgs {
        bidder_position,
        channel: channel(),
        channel_epoch,
        epoch: epoch(),
        now,
        credits_to_deposit,
    })
}

/// The first deposit on a new position
pub fn first_deposit(
    channel_epoch: ChannelEpoch<Id, Number>,
    now: Seconds,
    credits_to_deposit: Amount,
) -> Result<DepositCreditsReceipt<Id, Number>> {
    let position = empty_position(&channel_epoch);
    deposit(position, channel_epoch, now, credits_to_deposit)
}

pub fn shares(whole: u64) -> Shares<Number> {
    NonNeg::from_u64(whole).unwrap()
}

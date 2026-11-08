//! Fixtures shared by the action unit tests.

use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::models::Identifiable;
use crate::models::bidder_position::BidderPosition;
use crate::models::channel::Channel;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::channel_group::ChannelGroup;
use crate::models::epoch::Epoch;
use crate::{
    Amount, CreateBidderPositionArgs, CreateChannelArgs, CreateChannelReceipt, DepositCreditsArgs,
    DepositCreditsReceipt, EpochTerms, Result, Seconds, Shares, create_bidder_position,
    create_channel,
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

pub fn channel_group() -> ChannelGroup<Id> {
    ChannelGroup {
        id: CHANNEL_GROUP,
        master_start_time_unix_seconds: START - 3 * DURATION,
        epoch_duration_seconds: DURATION,
        current_epoch_index: 3,
    }
}

pub fn terms(ramp_bonus_slope: i32, exit_penalty_exponent: u32) -> EpochTerms<Number> {
    let slope = Number::from_i32(ramp_bonus_slope).unwrap();
    EpochTerms::new(slope, exit_penalty_exponent).unwrap()
}

/// A channel in the epoch's channel group, with the wiki's reference terms
pub fn channel() -> Channel<Id, Number> {
    created_channel(terms(1, 2)).new_channel
}

fn created_channel(default_terms: EpochTerms<Number>) -> CreateChannelReceipt<Id, Number> {
    create_channel(CreateChannelArgs {
        channel_id: CHANNEL,
        reward_id: Id(5),
        default_terms,
        channel_group: channel_group(),
        active_epoch: epoch(),
        now: START,
    })
    .unwrap()
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
    created_channel(terms(ramp_bonus_slope, exit_penalty_exponent)).new_channel_epoch
}

pub fn empty_position(channel_epoch: &ChannelEpoch<Id, Number>) -> BidderPosition<Id, Number> {
    // Positions already exist when a test later supplies a finalized epoch.
    let initial_epoch = ChannelEpoch::new(
        channel_epoch.channel_id,
        channel_epoch.epoch_counter,
        channel_epoch.terms,
    );
    create_bidder_position(CreateBidderPositionArgs {
        position_id: BIDDER,
        channel_group: channel_group(),
        channel: channel(),
        channel_epoch: initial_epoch,
        epoch: epoch(),
        now: START,
    })
    .unwrap()
    .bidder_position
}

pub fn deposit(
    bidder_position: BidderPosition<Id, Number>,
    channel_epoch: ChannelEpoch<Id, Number>,
    now: Seconds,
    credits_to_deposit: Amount,
) -> Result<DepositCreditsReceipt<Id, Number>> {
    crate::deposit_credits(DepositCreditsArgs {
        channel_group: channel_group(),
        bidder_position,
        channel: channel(),
        channel_epoch,
        epoch: epoch(),
        now,
        credits_to_deposit,
        max_lot_count: 8,
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

use mayf_fixed_number::FixedNumber;

use crate::Amount;
use crate::Result;
use crate::Seconds;
use crate::models::Identifiable;
use crate::models::bidder_position::BidderPosition;
use crate::models::channel_epoch::ChannelEpoch;
use crate::models::epoch::Epoch;

pub struct DepositCreditsArgs<I: Identifiable, N: FixedNumber> {
    pub bidder_position: BidderPosition<I, N>,
    pub channel_epoch: ChannelEpoch<I, N>,
    pub epoch: Epoch<I>,
    pub now: Seconds,
    pub credits_to_deposit: Amount,
}

pub struct DepositCreditsReceipt<I: Identifiable, N: FixedNumber> {
    pub bidder_position_next: BidderPosition<I, N>,
    pub channel_epoch_next: ChannelEpoch<I, N>,
}

pub fn deposit_credits<N, I>(args: DepositCreditsArgs<I, N>) -> Result<DepositCreditsReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    // validate things
    // append a new lot
    // update the channel epoch
    // return the next state

    todo!()
}

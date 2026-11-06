use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::Amount;
use crate::models::Identifiable;
use crate::models::channel_epoch::ChannelEpochId;

pub struct BidderPosition<I: Identifiable, N: FixedNumber> {
    pub id: I,
    pub channel_epoch_id: ChannelEpochId<I>,
    pub lots: Vec<BidderLot<I, N>>,
}

pub struct BidderLot<I: Identifiable, N: FixedNumber> {
    pub bidder_position_id: I,
    pub credits_deposited: Amount,
    pub entry_weight: NonNeg<N>,
    pub shares: NonNeg<N>,
}

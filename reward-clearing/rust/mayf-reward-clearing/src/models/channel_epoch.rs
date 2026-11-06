use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::{EpochTerms, models::Identifiable};

pub struct ChannelEpoch<I: Identifiable, N: FixedNumber> {
    pub channel_id: I,
    pub epoch_id: I,
    pub reward_pot_balance: u64,
    pub clearing_credits_balance: u64,
    pub weighted_shares_balance: NonNeg<N>,
    pub terms: EpochTerms<N>,
}

impl<I: Identifiable, N: FixedNumber> ChannelEpoch<I, N> {
    pub fn new(channel_id: I, epoch_id: I, terms: EpochTerms<N>) -> Self {
        Self {
            channel_id,
            epoch_id,
            reward_pot_balance: 0,
            clearing_credits_balance: 0,
            weighted_shares_balance: NonNeg::zero(),
            terms,
        }
    }
}

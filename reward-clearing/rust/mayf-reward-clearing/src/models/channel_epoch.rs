use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::error::Result;
use crate::models::epoch::Epoch;
use crate::{Amount, EpochTerms, models::Identifiable};
use crate::{EpochCounter, RewardClearingError};

pub struct ChannelEpoch<I: Identifiable, N: FixedNumber> {
    pub channel_id: I,
    pub epoch_counter: EpochCounter,
    pub reward_pot_balance: u64,
    pub clearing_credits_balance: u64,
    pub weighted_shares_balance: NonNeg<N>,
    pub terms: EpochTerms<N>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ChannelEpochId<I: Identifiable> {
    pub channel_id: I,
    pub epoch_counter: EpochCounter,
}

impl<I: Identifiable> Identifiable for ChannelEpochId<I> {}

impl<I: Identifiable, N: FixedNumber> ChannelEpoch<I, N> {
    pub fn id(&self) -> ChannelEpochId<I> {
        ChannelEpochId {
            channel_id: self.channel_id,
            epoch_counter: self.epoch_counter,
        }
    }

    pub fn new(channel_id: I, epoch_counter: EpochCounter, terms: EpochTerms<N>) -> Self {
        Self {
            channel_id,
            epoch_counter,
            reward_pot_balance: 0,
            clearing_credits_balance: 0,
            weighted_shares_balance: NonNeg::zero(),
            terms,
        }
    }

    pub fn add_reward(mut self, amount: Amount) -> Result<Self> {
        self.reward_pot_balance = self
            .reward_pot_balance
            .checked_add(amount)
            .ok_or(RewardClearingError::ArithmeticFailure)?;
        Ok(self)
    }

    pub fn validate_epoch(&self, epoch: &Epoch<I>) -> Result<()> {
        if self.epoch_counter != epoch.epoch_counter {
            return Err(RewardClearingError::ChannelEpochInvalidEpochCounter);
        }

        Ok(())
    }
}

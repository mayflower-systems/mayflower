use mayf_fixed_number::FixedNumber;

use crate::EpochTerms;
use crate::models::Identifiable;
use crate::models::epoch::Epoch;
use crate::{EpochCounter, Result, RewardClearingError};

/// A reward channel. Its epochs follow its channel group's schedule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Channel<I: Identifiable, N: FixedNumber> {
    pub id: I,
    pub channel_group_id: I,
    pub reward_id: I,
    pub default_terms: EpochTerms<N>,

    /// Disabling a channel prevents new epochs without affecting existing ones.
    pub disabled: bool,

    /// The epoch counter of the newest channel epoch this channel created.
    /// Channel creation opens its first epoch in the same action.
    /// A new channel epoch needs a later counter, so the channel never opens two for one epoch.
    pub last_created_epoch_counter: EpochCounter,
}

impl<I: Identifiable, N: FixedNumber> Channel<I, N> {
    pub fn validate_epoch(&self, epoch: &Epoch<I>) -> Result<()> {
        if self.channel_group_id != epoch.channel_group_id {
            return Err(RewardClearingError::ChannelGroupMismatch);
        }

        Ok(())
    }

    /// Record that this channel opened a channel epoch for `epoch`.
    /// Fails unless `epoch` is later than the last epoch the channel opened one for.
    pub fn record_channel_epoch(mut self, epoch: &Epoch<I>) -> Result<Self> {
        if epoch.epoch_counter <= self.last_created_epoch_counter {
            return Err(RewardClearingError::ChannelEpochAlreadyCreated);
        }

        self.last_created_epoch_counter = epoch.epoch_counter;
        Ok(self)
    }
}

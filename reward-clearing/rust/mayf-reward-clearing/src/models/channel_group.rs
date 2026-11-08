use crate::{EpochCounter, Result, RewardClearingError, Seconds, models::Identifiable};

/// Channels that share one epoch schedule
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelGroup<I: Identifiable> {
    pub id: I,
    pub master_start_time_unix_seconds: Seconds,
    pub epoch_duration_seconds: Seconds,
    pub current_epoch_index: EpochCounter,
}

impl<I: Identifiable> ChannelGroup<I> {
    pub fn next_epoch_index(&self) -> Result<EpochCounter> {
        self.current_epoch_index
            .checked_add(1)
            .ok_or(RewardClearingError::EpochCounterOverflow)
    }
}

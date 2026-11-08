use crate::models::channel_group::ChannelGroup;
use crate::models::{EpochCounter, Identifiable, Seconds};
use crate::{Result, RewardClearingError};

/// One epoch of a channel group's schedule
/// Deposits, withdrawals, and funding are accepted from its start up to, but not including, its end.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Epoch<I: Identifiable> {
    pub channel_group_id: I,
    pub epoch_counter: EpochCounter,
    pub start_time_unix_seconds: Seconds,
    pub duration_seconds: Seconds,
}

impl<I: Identifiable> Epoch<I> {
    pub fn new(
        channel_group_id: I,
        epoch_counter: EpochCounter,
        start_time_unix_seconds: Seconds,
        duration_seconds: Seconds,
    ) -> Self {
        Self {
            channel_group_id,
            epoch_counter,
            start_time_unix_seconds,
            duration_seconds,
        }
    }

    pub fn validate_channel_group(&self, channel_group: &ChannelGroup<I>) -> Result<()> {
        if self.channel_group_id != channel_group.id {
            return Err(RewardClearingError::EpochChannelGroupMismatch);
        }

        Ok(())
    }

    /// Fails unless this is the channel group's current epoch.
    pub fn validate_current(&self, channel_group: &ChannelGroup<I>) -> Result<()> {
        self.validate_channel_group(channel_group)?;
        if self.epoch_counter != channel_group.current_epoch_index {
            return Err(RewardClearingError::ChannelGroupCounterMismatch);
        }
        Ok(())
    }

    pub fn end_time_unix_seconds(&self) -> Result<Seconds> {
        self.start_time_unix_seconds
            .checked_add(self.duration_seconds)
            .ok_or(RewardClearingError::EpochEndTimeOverflow)
    }

    pub fn is_finished(&self, now: Seconds) -> Result<bool> {
        Ok(self.end_time_unix_seconds()? <= now)
    }

    /// Fails unless this is the group's current epoch and `now` is in [start, end).
    pub fn validate_active(&self, channel_group: &ChannelGroup<I>, now: Seconds) -> Result<()> {
        self.validate_current(channel_group)?;
        self.validate_live_interval(now)
    }

    fn validate_live_interval(&self, now: Seconds) -> Result<()> {
        if now < self.start_time_unix_seconds {
            return Err(RewardClearingError::EpochNotStarted);
        }

        if self.is_finished(now)? {
            return Err(RewardClearingError::EpochEnded);
        }

        Ok(())
    }

    /// Seconds since the epoch started. Fails outside the live interval [start, end).
    pub fn elapsed_seconds(&self, now: Seconds) -> Result<Seconds> {
        self.validate_live_interval(now)?;
        Ok(now - self.start_time_unix_seconds)
    }
}

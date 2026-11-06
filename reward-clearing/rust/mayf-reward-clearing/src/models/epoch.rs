use crate::{EpochCounter, Seconds, models::Identifiable};

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

    pub fn end_time_unix_seconds(&self) -> Seconds {
        self.start_time_unix_seconds
            .checked_add(self.duration_seconds)
            .unwrap()
    }

    pub fn is_finished(&self, now: Seconds) -> bool {
        self.end_time_unix_seconds() <= now
    }

    pub fn is_active(&self, now: Seconds) -> bool {
        !self.is_finished(now) && self.start_time_unix_seconds <= now
    }
}

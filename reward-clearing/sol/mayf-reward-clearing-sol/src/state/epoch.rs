use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Epoch {
    pub tenant: Pubkey,
    pub epoch_counter: u32,
    pub start_time_unix_seconds: u64,
    pub duration_seconds: u32,
}

impl Epoch {
    pub fn end_time_unix_seconds(&self) -> u64 {
        self.start_time_unix_seconds
            .checked_add(self.duration_seconds.into())
            .unwrap()
    }

    pub fn is_finished(&self, now: u64) -> bool {
        let end_time = self.end_time_unix_seconds();
        now >= end_time
    }

    pub fn is_active(&self, now: u64) -> bool {
        !self.is_finished(now) && now >= self.start_time_unix_seconds
    }
}

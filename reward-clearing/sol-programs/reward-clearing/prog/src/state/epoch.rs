use anchor_lang::prelude::*;
use mayf_reward_clearing::models::epoch::Epoch as LibEpoch;

use crate::{error::RewardClearingProgramError, state::com::AccountId};

#[account]
#[derive(InitSpace)]
pub struct Epoch {
    pub channel_group: Pubkey,
    pub epoch_counter: u32,
    pub start_time_unix_seconds: u64,
    pub duration_seconds: u32,
}

impl From<&Epoch> for LibEpoch<AccountId> {
    fn from(epoch: &Epoch) -> Self {
        Self {
            channel_group_id: AccountId(epoch.channel_group),
            epoch_counter: epoch.epoch_counter,
            start_time_unix_seconds: epoch.start_time_unix_seconds,
            duration_seconds: epoch.duration_seconds.into(),
        }
    }
}

impl TryFrom<LibEpoch<AccountId>> for Epoch {
    type Error = RewardClearingProgramError;

    fn try_from(epoch: LibEpoch<AccountId>) -> core::result::Result<Self, Self::Error> {
        Ok(Self {
            channel_group: epoch.channel_group_id.0,
            epoch_counter: epoch.epoch_counter,
            start_time_unix_seconds: epoch.start_time_unix_seconds,
            duration_seconds: epoch
                .duration_seconds
                .try_into()
                .map_err(|_| RewardClearingProgramError::EpochDurationOutOfRange)?,
        })
    }
}

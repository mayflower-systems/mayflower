use anchor_lang::prelude::*;

use crate::error::WeightedEmitterProgramError;

pub fn now_unix_seconds() -> Result<u64> {
    Clock::get()?
        .unix_timestamp
        .try_into()
        .map_err(|_| error!(WeightedEmitterProgramError::InvalidTimestamp))
}

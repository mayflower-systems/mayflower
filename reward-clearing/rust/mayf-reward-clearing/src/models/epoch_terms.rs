use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::{Result, RewardClearingError};

/// The entry and exit curve terms snapshotted when a channel epoch opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochTerms<N: FixedNumber> {
    ramp_bonus_slope: NonNeg<N>,
    exit_penalty_exponent: u32,
}

impl<N: FixedNumber> EpochTerms<N> {
    /// The largest ramp bonus slope, giving a starting entry weight of 3
    pub const MAX_RAMP_BONUS_SLOPE: i32 = 2;

    pub const MIN_EXIT_PENALTY_EXPONENT: u32 = 1;
    pub const MAX_EXIT_PENALTY_EXPONENT: u32 = 4;

    /// Set an entry bonus slope in `[0, 2]` and an exit exponent from 1 to 4.
    ///
    /// A zero slope gives every deposit unit weight. An exit exponent of 1
    /// gives a linear penalty, and 2 gives a quadratic penalty.
    pub fn new(ramp_bonus_slope: N, exit_penalty_exponent: u32) -> Result<Self> {
        let max_slope = N::from_i32(Self::MAX_RAMP_BONUS_SLOPE)
            .map_err(|_| RewardClearingError::InvalidRampBonusSlope)?;
        if ramp_bonus_slope > max_slope {
            return Err(RewardClearingError::InvalidRampBonusSlope);
        }
        let ramp_bonus_slope = NonNeg::new(ramp_bonus_slope)
            .map_err(|_| RewardClearingError::InvalidRampBonusSlope)?;

        let exponents = Self::MIN_EXIT_PENALTY_EXPONENT..=Self::MAX_EXIT_PENALTY_EXPONENT;
        if !exponents.contains(&exit_penalty_exponent) {
            return Err(RewardClearingError::InvalidExitPenaltyExponent);
        }

        Ok(Self {
            ramp_bonus_slope,
            exit_penalty_exponent,
        })
    }

    pub fn ramp_bonus_slope(&self) -> NonNeg<N> {
        self.ramp_bonus_slope
    }

    pub fn exit_penalty_exponent(&self) -> u32 {
        self.exit_penalty_exponent
    }
}

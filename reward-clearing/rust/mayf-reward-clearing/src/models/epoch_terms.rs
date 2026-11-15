use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::{Result, auction::AuctionTerms};

/// The entry and exit curve terms snapshotted when a channel epoch opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochTerms<N: FixedNumber>(pub(crate) AuctionTerms<N>);

impl<N: FixedNumber> EpochTerms<N> {
    /// The largest ramp bonus slope, giving a starting entry weight of 3
    pub const MAX_RAMP_BONUS_SLOPE: i32 = AuctionTerms::<N>::MAX_RAMP_BONUS_SLOPE;

    pub const MIN_EXIT_PENALTY_EXPONENT: u32 = AuctionTerms::<N>::MIN_EXIT_PENALTY_EXPONENT;
    pub const MAX_EXIT_PENALTY_EXPONENT: u32 = AuctionTerms::<N>::MAX_EXIT_PENALTY_EXPONENT;

    /// Set an entry bonus slope in `[0, 2]` and an exit exponent from 1 to 4.
    ///
    /// A zero slope gives every deposit unit weight. An exit exponent of 1
    /// gives a linear penalty, and 2 gives a quadratic penalty.
    pub fn new(ramp_bonus_slope: N, exit_penalty_exponent: u32) -> Result<Self> {
        Ok(Self(AuctionTerms::new(
            ramp_bonus_slope,
            exit_penalty_exponent,
        )?))
    }

    pub fn ramp_bonus_slope(&self) -> NonNeg<N> {
        self.0.ramp_bonus_slope()
    }

    pub fn exit_penalty_exponent(&self) -> u32 {
        self.0.exit_penalty_exponent()
    }
}

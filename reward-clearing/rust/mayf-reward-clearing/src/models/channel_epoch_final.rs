use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::models::Identifiable;
use crate::models::channel_epoch::{ChannelEpoch, ChannelEpochId};
use crate::{Result, RewardClearingError};

/// A finalized channel epoch
/// The channel epoch's reward pot and weighted shares stop changing when its epoch ends.
/// They are the numerator and denominator of every claim, so they stay on the channel epoch.
pub struct ChannelEpochFinal<I: Identifiable, N: FixedNumber> {
    pub channel_epoch_id: ChannelEpochId<I>,

    /// Weighted shares that have not claimed yet
    /// Each claim subtracts its shares. At zero, the remaining reward can go to the protocol.
    pub unclaimed_weighted_shares: NonNeg<N>,
}

impl<I: Identifiable, N: FixedNumber> ChannelEpochFinal<I, N> {
    pub fn validate_channel_epoch(&self, channel_epoch: &ChannelEpoch<I, N>) -> Result<()> {
        if self.channel_epoch_id != channel_epoch.id() {
            return Err(RewardClearingError::ChannelEpochFinalMismatch);
        }

        Ok(())
    }

    /// Subtract a claim's shares from the unclaimed shares.
    pub fn claim_shares(mut self, shares: NonNeg<N>) -> Result<Self> {
        self.unclaimed_weighted_shares = self
            .unclaimed_weighted_shares
            .checked_sub(shares)
            .map_err(|_| RewardClearingError::UnclaimedSharesUnderflow)?;
        Ok(self)
    }
}

use core::num::NonZeroU64;

use mayf_fixed_number::{DirectedRounding, FixedNumber, IntegralRounding, NonNeg, Pos};

use crate::models::epoch::Epoch;
use crate::{Amount, EpochTerms, models::Identifiable};
use crate::{EpochCounter, Result, RewardClearingError, Seconds, Shares};

/// A channel's epoch
/// There is one ChannelEpoch for each channel for each epoch
/// This tracks a cumulative pot of rewards to be distributed out to the bidders when it is finalized
pub struct ChannelEpoch<I: Identifiable, N: FixedNumber> {
    pub channel_id: I,
    pub epoch_counter: EpochCounter,
    pub reward_pot_balance: Amount,
    pub clearing_credits_balance: Amount,
    pub weighted_shares_balance: Shares<N>,
    pub terms: EpochTerms<N>,
}

/// Identifies a channel epoch by its channel and epoch counter
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ChannelEpochId<I: Identifiable> {
    pub channel_id: I,
    pub epoch_counter: EpochCounter,
}

impl<I: Identifiable> Identifiable for ChannelEpochId<I> {}

impl<I: Identifiable, N: FixedNumber> ChannelEpoch<I, N> {
    pub fn id(&self) -> ChannelEpochId<I> {
        ChannelEpochId {
            channel_id: self.channel_id,
            epoch_counter: self.epoch_counter,
        }
    }

    pub fn new(channel_id: I, epoch_counter: EpochCounter, terms: EpochTerms<N>) -> Self {
        Self {
            channel_id,
            epoch_counter,
            reward_pot_balance: 0,
            clearing_credits_balance: 0,
            weighted_shares_balance: Shares::zero(),
            terms,
        }
    }

    pub fn add_reward(mut self, amount: Amount) -> Result<Self> {
        self.reward_pot_balance = self
            .reward_pot_balance
            .checked_add(amount)
            .ok_or(RewardClearingError::RewardPotOverflow)?;
        Ok(self)
    }

    /// Add a new lot's credits and weighted shares to the pot's balances.
    pub fn deposit_credits(mut self, credits: Amount, weighted_shares: Shares<N>) -> Result<Self> {
        self.clearing_credits_balance = self
            .clearing_credits_balance
            .checked_add(credits)
            .ok_or(RewardClearingError::CreditBalanceOverflow)?;
        self.weighted_shares_balance = self
            .weighted_shares_balance
            .checked_add(weighted_shares)
            .map_err(|_| RewardClearingError::WeightedSharesOverflow)?;
        Ok(self)
    }

    /// Remove withdrawn credits and their weighted shares from the pot's balances.
    pub fn withdraw_credits(mut self, credits: Amount, weighted_shares: Shares<N>) -> Result<Self> {
        self.clearing_credits_balance = self
            .clearing_credits_balance
            .checked_sub(credits)
            .ok_or(RewardClearingError::CreditBalanceUnderflow)?;
        self.weighted_shares_balance = self
            .weighted_shares_balance
            .checked_sub(weighted_shares)
            .map_err(|_| RewardClearingError::WeightedSharesUnderflow)?;
        Ok(self)
    }

    /// The entry weight for credits deposited at `now`:
    ///
    ///   w = 1 + floor(slope * floor((duration - elapsed) / duration))
    ///
    /// It starts at `1 + slope` when the epoch opens and falls linearly toward 1.
    /// Both steps round down, so rounding never issues extra shares.
    pub fn entry_weight(&self, epoch: &Epoch<I>, now: Seconds) -> Result<NonNeg<N>> {
        let elapsed = epoch.elapsed_seconds(now)?;
        let duration = NonZeroU64::new(epoch.duration_seconds)
            .ok_or(RewardClearingError::InvalidEpochDuration)?;
        let remaining = duration.get() - elapsed;

        let remaining_fraction =
            N::checked_from_ratio_round(remaining.into(), duration.into(), DirectedRounding::Floor)
                .and_then(NonNeg::new)
                .map_err(|_| RewardClearingError::EntryWeightOverflow)?;
        let bonus = self
            .terms
            .ramp_bonus_slope()
            .checked_mul_floor(remaining_fraction)
            .map_err(|_| RewardClearingError::EntryWeightOverflow)?;

        NonNeg::one()
            .checked_add(bonus)
            .map_err(|_| RewardClearingError::EntryWeightOverflow)
    }

    /// The fraction of withdrawn credits forfeited at `now`:
    ///
    ///   p = min(1, ceil(elapsed / duration) ^ exponent)
    ///
    /// It starts at 0 when the epoch opens and rises toward 1.
    /// Every step rounds up, so rounding never returns extra credits.
    pub fn exit_penalty(&self, epoch: &Epoch<I>, now: Seconds) -> Result<NonNeg<N>> {
        let elapsed = epoch.elapsed_seconds(now)?;
        let duration = NonZeroU64::new(epoch.duration_seconds)
            .ok_or(RewardClearingError::InvalidEpochDuration)?;

        let penalty =
            N::checked_from_ratio_round(elapsed.into(), duration.into(), DirectedRounding::Ceil)
                .and_then(|progress| progress.checked_pow_ceil(self.terms.exit_penalty_exponent()))
                .and_then(NonNeg::new)
                .map_err(|_| RewardClearingError::ExitPenaltyOverflow)?;

        Ok(penalty.min(NonNeg::one()))
    }

    pub fn validate_epoch(&self, epoch: &Epoch<I>) -> Result<()> {
        if self.epoch_counter != epoch.epoch_counter {
            return Err(RewardClearingError::ChannelEpochCounterMismatch);
        }

        Ok(())
    }
}

impl<I: Identifiable, N: IntegralRounding> ChannelEpoch<I, N> {
    /// The reward for claimed shares, rounded down to whole units:
    ///
    ///   reward = floor(reward_pot * shares / weighted_shares)
    ///
    /// The pot and shares stop changing when the epoch ends,
    /// so every claim uses the same ratio and claim order does not change payouts.
    pub fn claim_reward(&self, shares: Shares<N>) -> Result<Amount> {
        let weighted_shares = Pos::try_from(self.weighted_shares_balance)
            .map_err(|_| RewardClearingError::ZeroWeightedShares)?;

        NonNeg::<N>::from_u64(self.reward_pot_balance)
            .and_then(|pot| pot.checked_mul_div_floor(shares, weighted_shares))
            .and_then(|reward| reward.get().checked_floor_to_u64())
            .map_err(|_| RewardClearingError::ClaimRewardOverflow)
    }
}

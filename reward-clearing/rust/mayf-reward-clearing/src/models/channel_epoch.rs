use core::num::NonZeroU64;

use mayf_fixed_number::{DirectedRounding, FixedNumber, IntegralRounding, NonNeg, Pos};

use crate::models::channel::Channel;
use crate::models::epoch::Epoch;
use crate::{Amount, EpochTerms, models::Identifiable};
use crate::{EpochCounter, Result, RewardClearingError, Seconds, Shares};

/// A channel's epoch
/// There is at most one ChannelEpoch for each channel and epoch
/// This tracks a cumulative pot of rewards to be distributed out to the bidders when it is finalized
///
/// After finalization, each claim pays from `reward_pot_balance` and retires its shares
/// from `weighted_shares_balance`. Once no shares remain, the rest of the pot goes to the protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelEpoch<I: Identifiable, N: FixedNumber> {
    pub channel_id: I,
    pub epoch_counter: EpochCounter,
    pub reward_pot_balance: Amount,
    pub clearing_credits_balance: Amount,
    pub weighted_shares_balance: Shares<N>,
    pub terms: EpochTerms<N>,

    /// Totals fixed once by finalization and used by every claim. `None` until then.
    pub final_totals: Option<FinalizedTotals<N>>,
}

/// The immutable numerator and denominator for every claim in a finalized epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinalizedTotals<N: FixedNumber> {
    pub reward_pot: Amount,
    pub weighted_shares: Shares<N>,
}

/// Identifies a channel epoch by its channel and epoch counter
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
            final_totals: None,
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

    /// Freeze the reward pot and total shares for all subsequent claims.
    /// An epoch with no shares still finalizes; its whole reward pot is protocol remainder.
    pub fn finalize(mut self) -> Result<Self> {
        self.validate_unfinalized()?;

        self.final_totals = Some(FinalizedTotals {
            reward_pot: self.reward_pot_balance,
            weighted_shares: self.weighted_shares_balance,
        });
        Ok(self)
    }

    /// A finalized epoch cannot accept funding or changes to its committed credits.
    pub fn validate_unfinalized(&self) -> Result<()> {
        if self.final_totals.is_some() {
            return Err(RewardClearingError::ChannelEpochAlreadyFinalized);
        }
        Ok(())
    }

    /// Fails unless this is the channel's channel epoch for `epoch`.
    /// The epoch counter alone is ambiguous across channel groups,
    /// so the channel links this channel epoch to the epoch's channel group.
    pub fn validate_channel_and_epoch(
        &self,
        channel: &Channel<I, N>,
        epoch: &Epoch<I>,
    ) -> Result<()> {
        if self.channel_id != channel.id {
            return Err(RewardClearingError::ChannelEpochChannelMismatch);
        }

        channel.validate_epoch(epoch)?;

        if self.epoch_counter != epoch.epoch_counter {
            return Err(RewardClearingError::ChannelEpochCounterMismatch);
        }

        Ok(())
    }
}

impl<I: Identifiable, N: IntegralRounding> ChannelEpoch<I, N> {
    /// Pay a claim against the frozen totals, rounded down to whole units:
    ///
    ///   reward = floor(final_reward_pot * shares / final_weighted_shares)
    ///
    /// Multiply and divide together without rounding a per-share rate first.
    /// Every claim uses the same totals, so claim order does not change payouts.
    /// Returns the next channel epoch and the reward earned.
    pub fn claim(mut self, shares: Shares<N>) -> Result<(Self, Amount)> {
        let totals = self
            .final_totals
            .ok_or(RewardClearingError::ChannelEpochNotFinalized)?;

        self.weighted_shares_balance = self
            .weighted_shares_balance
            .checked_sub(shares)
            .map_err(|_| RewardClearingError::WeightedSharesUnderflow)?;

        let total_shares = Pos::try_from(totals.weighted_shares)
            .map_err(|_| RewardClearingError::NoSharesToClaim)?;
        let reward = NonNeg::<N>::from_u64(totals.reward_pot)
            .and_then(|pot| pot.checked_mul_div_floor(shares, total_shares))
            .and_then(|reward| reward.get().checked_floor_to_u64())
            .map_err(|_| RewardClearingError::ClaimRewardOverflow)?;

        self.reward_pot_balance = self
            .reward_pot_balance
            .checked_sub(reward)
            .ok_or(RewardClearingError::RewardPotUnderflow)?;
        Ok((self, reward))
    }
}

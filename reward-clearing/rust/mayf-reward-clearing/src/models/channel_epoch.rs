use mayf_auction_calculator::AuctionWindow;
use mayf_fixed_number::{FixedNumber, IntegralRounding, NonNeg, Pos};

use crate::models::channel::Channel;
use crate::models::epoch::Epoch;
use crate::{Amount, EpochTerms, models::Identifiable};
use crate::{EpochCounter, Result, RewardClearingError, Seconds, Shares};

/// One auction for a channel in a group epoch.
///
/// After finalization, claims retire shares and draw from the reward balance.
/// Any reward left after the last claim remains recorded in the pot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelEpoch<I: Identifiable, N: FixedNumber> {
    pub channel_id: I,
    pub epoch_counter: EpochCounter,
    /// Accepted funding minus payouts already claimed.
    pub reward_pot_balance: Amount,
    /// Committed principal before finalization, historical burn amount afterward.
    pub clearing_credits_balance: Amount,
    /// Surviving shares before finalization, unclaimed shares afterward.
    pub weighted_shares_balance: Shares<N>,
    /// The channel terms copied when this auction opened.
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

    /// The entry weight for credits deposited at `now`.
    ///
    /// The ideal weight is `1 + slope * (1 - elapsed / duration)`, where
    /// elapsed time starts at the group epoch's start. This rounds the
    /// remaining time fraction and its product with the slope down to the
    /// fixed-point grid. The deadline is outside the live interval.
    pub fn entry_weight(&self, epoch: &Epoch<I>, now: Seconds) -> Result<NonNeg<N>> {
        Ok(self.auction_window(epoch).entry(now)?.get())
    }

    /// The fraction of withdrawn credits forfeited at `now`.
    ///
    /// The ideal fraction is `(elapsed / duration)^exponent`. Elapsed time is
    /// measured from the group epoch's start for every lot. The elapsed
    /// fraction and each multiplication in the power round up to the grid.
    /// The result is capped at one.
    pub fn exit_penalty(&self, epoch: &Epoch<I>, now: Seconds) -> Result<NonNeg<N>> {
        Ok(self.auction_window(epoch).exit_penalty(now)?)
    }

    pub(crate) fn auction_window(&self, epoch: &Epoch<I>) -> AuctionWindow<N> {
        AuctionWindow::new(epoch.window(), self.terms.0)
    }

    /// Freeze the reward pot and total shares for all subsequent claims.
    /// An epoch with no shares still finalizes and retains its entire reward pot.
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
    /// Pay `floor(final_reward_pot * shares / final_weighted_shares)`.
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

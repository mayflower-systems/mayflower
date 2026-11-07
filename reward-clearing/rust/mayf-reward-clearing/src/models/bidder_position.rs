use mayf_fixed_number::{FixedNumber, NonNeg, Pos};

use crate::models::Identifiable;
use crate::models::channel_epoch::{ChannelEpoch, ChannelEpochId};
use crate::{Amount, Result, RewardClearingError};

/// A position for a bidder on a channel's epoch
pub struct BidderPosition<I: Identifiable, N: FixedNumber> {
    pub id: I,
    pub channel_epoch_id: ChannelEpochId<I>,

    /// A stack of lots, one per deposit. Withdrawals take from the newest lot first.
    /// Entry weight falls through the epoch, so the newest lot has the fewest shares per credit.
    pub lots: Vec<BidderLot<I, N>>,
}

/// The credits and shares left from one deposit
pub struct BidderLot<I: Identifiable, N: FixedNumber> {
    pub bidder_position_id: I,

    /// Credits remaining in this lot
    pub credits: Amount,

    /// Weighted shares remaining in this lot
    pub shares: NonNeg<N>,
}

impl<I: Identifiable, N: FixedNumber> BidderPosition<I, N> {
    pub fn validate_channel_epoch(&self, channel_epoch: &ChannelEpoch<I, N>) -> Result<()> {
        if self.channel_epoch_id != channel_epoch.id() {
            return Err(RewardClearingError::BidderPositionChannelEpochMismatch);
        }

        Ok(())
    }

    /// Push a new lot of credits at an entry weight.
    /// Returns the next position and the shares issued.
    pub fn deposit_credits(
        mut self,
        credits: Amount,
        entry_weight: NonNeg<N>,
    ) -> Result<(Self, NonNeg<N>)> {
        let lot = BidderLot::new(self.id, credits, entry_weight)?;
        let shares_issued = lot.shares;

        self.lots.push(lot);
        Ok((self, shares_issued))
    }

    /// Take credits from the newest lots first, popping each lot that empties.
    /// Returns the next position and the shares removed.
    pub fn withdraw_credits(mut self, credits: Amount) -> Result<(Self, NonNeg<N>)> {
        if credits == 0 {
            return Err(RewardClearingError::WithdrawalTooSmall);
        }

        let mut credits_left = credits;
        let mut shares_removed = NonNeg::zero();

        while credits_left > 0 {
            let lot = self
                .lots
                .pop()
                .ok_or(RewardClearingError::WithdrawalExceedsPositionCredits)?;

            let credits_from_lot = credits_left.min(lot.credits);
            let (lot_next, lot_shares_removed) = lot.withdraw(credits_from_lot)?;

            credits_left -= credits_from_lot;
            shares_removed = shares_removed
                .checked_add(lot_shares_removed)
                .map_err(|_| RewardClearingError::WithdrawnSharesOverflow)?;

            if let Some(lot_next) = lot_next {
                self.lots.push(lot_next);
            }
        }

        Ok((self, shares_removed))
    }

    /// Retire every lot's shares for a claim.
    /// Returns the next position, with no lots, and the shares claimed.
    pub fn claim(mut self) -> Result<(Self, NonNeg<N>)> {
        let mut shares_claimed = NonNeg::zero();

        for lot in self.lots.drain(..) {
            shares_claimed = shares_claimed
                .checked_add(lot.shares)
                .map_err(|_| RewardClearingError::ClaimedSharesOverflow)?;
        }

        if shares_claimed == NonNeg::zero() {
            return Err(RewardClearingError::NoSharesToClaim);
        }

        Ok((self, shares_claimed))
    }
}

impl<I: Identifiable, N: FixedNumber> BidderLot<I, N> {
    /// shares = floor(credits * entry_weight)
    fn new(bidder_position_id: I, credits: Amount, entry_weight: NonNeg<N>) -> Result<Self> {
        let shares = NonNeg::<N>::from_u64(credits)
            .and_then(|credits| credits.checked_mul_floor(entry_weight))
            .map_err(|_| RewardClearingError::LotSharesOverflow)?;

        if shares == NonNeg::zero() {
            return Err(RewardClearingError::DepositTooSmall);
        }

        Ok(Self {
            bidder_position_id,
            credits,
            shares,
        })
    }

    /// Remove credits and their share of the lot's shares.
    /// A partial withdrawal rounds the shares removed up:
    ///
    ///   shares_removed = min(shares, ceil(withdrawn * shares / credits))
    ///
    /// Returns the remaining lot, or `None` once it is empty, and the shares removed.
    fn withdraw(self, withdrawn: Amount) -> Result<(Option<Self>, NonNeg<N>)> {
        if withdrawn == self.credits {
            return Ok((None, self.shares));
        }

        let shares_removed = NonNeg::<N>::from_u64(self.credits)
            .and_then(Pos::try_from)
            .and_then(|credits| {
                NonNeg::<N>::from_u64(withdrawn)?.checked_mul_div_ceil(self.shares, credits)
            })
            .map_err(|_| RewardClearingError::LotSharesRemovedOverflow)?
            .min(self.shares);

        let lot_next = Self {
            bidder_position_id: self.bidder_position_id,
            credits: self
                .credits
                .checked_sub(withdrawn)
                .ok_or(RewardClearingError::LotCreditsUnderflow)?,
            shares: self
                .shares
                .checked_sub(shares_removed)
                .map_err(|_| RewardClearingError::LotSharesUnderflow)?,
        };

        Ok((Some(lot_next), shares_removed))
    }
}

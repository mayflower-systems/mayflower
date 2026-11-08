use mayf_fixed_number::{FixedNumber, NonNeg, Pos};

use crate::models::Identifiable;
use crate::models::channel_epoch::{ChannelEpoch, ChannelEpochId};
use crate::{Amount, Result, RewardClearingError, Shares};

/// A position for a bidder on a channel's epoch
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BidderPosition<I: Identifiable, N: FixedNumber> {
    pub id: I,
    pub channel_epoch_id: ChannelEpochId<I>,

    /// A stack of lots, one per deposit. Withdrawals take from the newest lot first.
    /// Entry weight falls through the epoch, so the newest lot has the fewest shares per credit.
    pub lots: Vec<BidderLot<I, N>>,

    /// Set by a claim. A claimed position is frozen: it cannot deposit, withdraw, or claim again.
    pub claimed: bool,
}

/// The credits and shares left from one deposit
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BidderLot<I: Identifiable, N: FixedNumber> {
    pub bidder_position_id: I,

    /// Credits remaining in this lot
    pub credits: Amount,

    /// Weighted shares remaining in this lot
    pub shares: Shares<N>,
}

impl<I: Identifiable, N: FixedNumber> BidderPosition<I, N> {
    pub fn new(id: I, channel_epoch_id: ChannelEpochId<I>) -> Self {
        Self {
            id,
            channel_epoch_id,
            lots: Vec::new(),
            claimed: false,
        }
    }

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
    ) -> Result<(Self, Shares<N>)> {
        self.validate()?;

        let lot = BidderLot::new(self.id, credits, entry_weight)?;
        let shares_issued = lot.shares;

        self.lots.push(lot);
        Ok((self, shares_issued))
    }

    /// Take credits from the newest lots first, popping each lot that empties.
    /// Returns the next position and the shares removed.
    pub fn withdraw_credits(mut self, credits: Amount) -> Result<(Self, Shares<N>)> {
        self.validate()?;

        if credits == 0 {
            return Err(RewardClearingError::WithdrawalTooSmall);
        }

        let mut credits_left = credits;
        let mut shares_removed = Shares::zero();

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

    /// Claim every lot's shares and freeze the position.
    /// Returns the next position and the shares claimed.
    pub fn claim(mut self) -> Result<(Self, Shares<N>)> {
        self.validate()?;

        let mut shares_claimed = Shares::zero();

        for lot in &self.lots {
            shares_claimed = shares_claimed
                .checked_add(lot.shares)
                .map_err(|_| RewardClearingError::ClaimedSharesOverflow)?;
        }

        if shares_claimed == NonNeg::zero() {
            return Err(RewardClearingError::NoSharesToClaim);
        }

        self.claimed = true;
        Ok((self, shares_claimed))
    }

    /// Fails if the position is frozen by a claim, or holds a lot from another position.
    fn validate(&self) -> Result<()> {
        if self.claimed {
            return Err(RewardClearingError::BidderPositionClaimed);
        }

        if self
            .lots
            .iter()
            .any(|lot| lot.bidder_position_id != self.id)
        {
            return Err(RewardClearingError::BidderLotPositionMismatch);
        }

        Ok(())
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
    fn withdraw(self, withdrawn: Amount) -> Result<(Option<Self>, Shares<N>)> {
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

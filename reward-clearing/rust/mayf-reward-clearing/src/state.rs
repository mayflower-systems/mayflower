//! Immutable reward-clearing state with validity encoded in its constructors.
//!
//! Domain values keep their fields opaque to callers. Public numeric snapshots
//! are validated on conversion; actions also check the accounting relationships
//! among an epoch, a position, and the supplied current lot.

use core::num::NonZeroU64;

use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::{Amount, Result, RewardClearingError};

pub(crate) fn validate_principal_share_bounds<N: FixedNumber>(
    principal: Amount,
    shares: NonNeg<N>,
    maximum_weight: NonNeg<N>,
    error: RewardClearingError,
) -> Result<()> {
    let principal_is_empty = principal == 0;
    let principal = crate::amount::from_amount(principal)?;
    if principal_is_empty != (shares == NonNeg::zero()) || shares < principal {
        return Err(error);
    }

    // Compare on the principal side: shares <= floor(principal * weight)
    // iff ceil(shares / weight) <= principal. The weight is at least one,
    // so this quotient fits even when the hypothetical maximum shares do not.
    let minimum_principal = shares
        .get()
        .checked_div_ceil(maximum_weight.get())
        .map_err(|_| RewardClearingError::ArithmeticFailure)?;
    if minimum_principal > principal.get() {
        return Err(error);
    }

    Ok(())
}

fn maximum_entry_weight<N: FixedNumber>() -> Result<NonNeg<N>> {
    let value = N::from_i32(1 + RampBonusSlope::<N>::MAX)
        .map_err(|_| RewardClearingError::ArithmeticFailure)?;
    NonNeg::new(value).map_err(|_| RewardClearingError::ArithmeticFailure)
}

fn maximum_entry_weight_for_slope<N: FixedNumber>(slope: RampBonusSlope<N>) -> Result<NonNeg<N>> {
    NonNeg::one()
        .checked_add(slope.get())
        .map_err(|_| RewardClearingError::ArithmeticFailure)
}

/// A timestamp in seconds.
pub type Seconds = u64;
/// Wall-clock epoch index derived from an immutable schedule.
pub type EpochCounter = u32;

/// A validated, strictly positive epoch duration.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EpochDuration(NonZeroU64);

impl EpochDuration {
    /// Validate and wrap a duration in seconds.
    pub const fn new(seconds: Seconds) -> Result<Self> {
        match NonZeroU64::new(seconds) {
            Some(seconds) => Ok(Self(seconds)),
            None => Err(RewardClearingError::InvalidEpochDuration),
        }
    }

    /// Return the validated duration in seconds.
    pub const fn get(self) -> Seconds {
        self.0.get()
    }
}

impl TryFrom<Seconds> for EpochDuration {
    type Error = RewardClearingError;

    fn try_from(seconds: Seconds) -> Result<Self> {
        Self::new(seconds)
    }
}

impl From<EpochDuration> for NonZeroU64 {
    fn from(duration: EpochDuration) -> Self {
        duration.0
    }
}

impl From<EpochDuration> for Seconds {
    fn from(duration: EpochDuration) -> Self {
        duration.get()
    }
}

/// Immutable wall-clock cadence from which every legal epoch window is
/// derived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochSchedule {
    master_start: Seconds,
    duration: EpochDuration,
}

impl EpochSchedule {
    /// Validate a schedule and prove that its first end is representable.
    pub fn try_new(master_start: Seconds, duration: Seconds) -> Result<Self> {
        let duration = EpochDuration::new(duration)?;
        master_start
            .checked_add(duration.get())
            .ok_or(RewardClearingError::EpochEndOverflow)?;
        Ok(Self {
            master_start,
            duration,
        })
    }

    pub const fn master_start(self) -> Seconds {
        self.master_start
    }

    pub const fn duration(self) -> EpochDuration {
        self.duration
    }

    /// Derive the unique schedule window containing `now`.
    pub fn window_at(self, now: Seconds) -> Result<EpochWindow> {
        let elapsed = now
            .checked_sub(self.master_start)
            .ok_or(RewardClearingError::ScheduleNotStarted)?;
        let counter = elapsed / self.duration.get();
        let counter = EpochCounter::try_from(counter)
            .map_err(|_| RewardClearingError::EpochCounterOverflow)?;
        self.window(counter)
    }

    /// Derive the end of an indexed window without exposing an arbitrary
    /// [`EpochWindow`] proof that could be passed to `start_epoch`.
    ///
    /// A caller that has only a schedule and a counter can recover when that
    /// epoch ended, without materializing a window it could then start.
    pub fn end_of(self, counter: EpochCounter) -> Result<Seconds> {
        self.window(counter).map(EpochWindow::end)
    }

    /// Derive one indexed window. Kept crate-private so numeric epoch
    /// snapshots can be checked without exposing an arbitrary-counter start
    /// path to callers of [`EpochSchedule::window_at`].
    pub(crate) fn window(self, counter: EpochCounter) -> Result<EpochWindow> {
        let offset = u64::from(counter)
            .checked_mul(self.duration.get())
            .ok_or(RewardClearingError::EpochEndOverflow)?;
        let start = self
            .master_start
            .checked_add(offset)
            .ok_or(RewardClearingError::EpochEndOverflow)?;
        let end = start
            .checked_add(self.duration.get())
            .ok_or(RewardClearingError::EpochEndOverflow)?;
        Ok(EpochWindow {
            schedule: self,
            counter,
            start,
            end,
        })
    }
}

/// A canonical indexed interval derived from an [`EpochSchedule`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochWindow {
    schedule: EpochSchedule,
    counter: EpochCounter,
    start: Seconds,
    end: Seconds,
}

impl EpochWindow {
    pub const fn counter(self) -> EpochCounter {
        self.counter
    }

    pub const fn end(self) -> Seconds {
        self.end
    }
}

/// A validated linear early-entry bonus in the inclusive range `[0, 2]`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RampBonusSlope<N: FixedNumber>(NonNeg<N>);

impl<N: FixedNumber> RampBonusSlope<N> {
    /// Largest permitted slope, producing a `3x` initial entry weight. The
    /// floor is structural: the wrapped value is `NonNeg`, so zero — a flat
    /// `1x` entry weight — is the smallest a slope can be.
    pub const MAX: i32 = 2;

    /// Validate and wrap a fixed-point slope.
    pub fn new(value: N) -> Result<Self> {
        let maximum =
            N::from_i32(Self::MAX).map_err(|_| RewardClearingError::InvalidRampBonusSlope)?;
        if value < N::ZERO || value > maximum {
            return Err(RewardClearingError::InvalidRampBonusSlope);
        }

        Ok(Self(
            NonNeg::new(value).map_err(|_| RewardClearingError::InvalidRampBonusSlope)?,
        ))
    }

    /// Return the validated non-negative slope.
    pub fn get(self) -> NonNeg<N> {
        self.0
    }
}

/// A validated withdrawal-penalty exponent in the inclusive range `[1, 4]`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExitPenaltyExponent(u32);

impl ExitPenaltyExponent {
    /// Smallest permitted exponent, producing a linear penalty.
    pub const MIN: u32 = 1;
    /// Largest permitted exponent, producing a quartic penalty.
    pub const MAX: u32 = 4;

    /// Validate and wrap an integer exponent.
    pub const fn new(value: u32) -> Result<Self> {
        if value < Self::MIN || value > Self::MAX {
            return Err(RewardClearingError::InvalidExitPenaltyExponent);
        }

        Ok(Self(value))
    }

    /// Return the validated exponent.
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Immutable curve and timing terms selected for one epoch.
///
/// Raw terms can enter this type only through [`EpochTerms::try_new`]. Once
/// created, values outside the protocol ranges cannot be represented.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochTerms<N: FixedNumber> {
    duration: EpochDuration,
    ramp_bonus_slope: RampBonusSlope<N>,
    exit_penalty_exponent: ExitPenaltyExponent,
}

impl<N: FixedNumber> EpochTerms<N> {
    /// Assemble already validated epoch terms.
    pub const fn new(
        duration: EpochDuration,
        ramp_bonus_slope: RampBonusSlope<N>,
        exit_penalty_exponent: ExitPenaltyExponent,
    ) -> Self {
        Self {
            duration,
            ramp_bonus_slope,
            exit_penalty_exponent,
        }
    }

    /// Validate raw curve and duration terms.
    pub fn try_new(
        duration: Seconds,
        ramp_bonus_slope: N,
        exit_penalty_exponent: u32,
    ) -> Result<Self> {
        Ok(Self::new(
            EpochDuration::new(duration)?,
            RampBonusSlope::new(ramp_bonus_slope)?,
            ExitPenaltyExponent::new(exit_penalty_exponent)?,
        ))
    }

    /// Concrete duration fixed for the epoch.
    pub const fn duration(self) -> EpochDuration {
        self.duration
    }

    /// Linear early-entry bonus fixed for the epoch.
    pub const fn ramp_bonus_slope(self) -> RampBonusSlope<N> {
        self.ramp_bonus_slope
    }

    /// Exit-penalty exponent fixed for the epoch.
    pub const fn exit_penalty_exponent(self) -> ExitPenaltyExponent {
        self.exit_penalty_exponent
    }

    /// Greatest entry weight permitted by these immutable terms.
    pub fn maximum_entry_weight(self) -> Result<NonNeg<N>> {
        maximum_entry_weight_for_slope(self.ramp_bonus_slope)
    }
}

/// Numeric inputs for validating an active epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpochSnapshot<N: FixedNumber> {
    pub master_start: Seconds,
    pub epoch_counter: EpochCounter,
    pub start: Seconds,
    pub duration: Seconds,
    pub ramp_bonus_slope: N,
    pub exit_penalty_exponent: u32,
    pub principal: Amount,
    pub unclaimed_shares: NonNeg<N>,
    pub unclaimed_reward: Amount,
}

/// Aggregate principal, shares, and net reward for one active auction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Epoch<N: FixedNumber> {
    pub(crate) master_start: Seconds,
    pub(crate) epoch_counter: EpochCounter,
    pub(crate) start: Seconds,
    pub(crate) end: Seconds,
    pub(crate) terms: EpochTerms<N>,
    pub(crate) principal: Amount,
    pub(crate) unclaimed_shares: NonNeg<N>,
    pub(crate) unclaimed_reward: Amount,
}

impl<N: FixedNumber> Epoch<N> {
    pub(crate) fn try_new(window: EpochWindow, terms: EpochTerms<N>) -> Result<Self> {
        if terms.duration() != window.schedule.duration() {
            return Err(RewardClearingError::EpochDurationMismatch);
        }
        Ok(Self {
            master_start: window.schedule.master_start(),
            epoch_counter: window.counter,
            start: window.start,
            end: window.end,
            terms,
            principal: 0,
            unclaimed_shares: NonNeg::zero(),
            unclaimed_reward: 0,
        })
    }

    pub const fn master_start(&self) -> Seconds {
        self.master_start
    }
    pub const fn epoch_counter(&self) -> EpochCounter {
        self.epoch_counter
    }
    pub const fn start(&self) -> Seconds {
        self.start
    }
    pub const fn end(&self) -> Seconds {
        self.end
    }
    pub const fn duration(&self) -> EpochDuration {
        self.terms.duration()
    }
    pub const fn terms(&self) -> EpochTerms<N> {
        self.terms
    }
    pub const fn ramp_bonus_slope(&self) -> RampBonusSlope<N> {
        self.terms.ramp_bonus_slope()
    }
    pub const fn exit_penalty_exponent(&self) -> ExitPenaltyExponent {
        self.terms.exit_penalty_exponent()
    }
    pub const fn principal(&self) -> Amount {
        self.principal
    }
    pub const fn unclaimed_shares(&self) -> NonNeg<N> {
        self.unclaimed_shares
    }
    pub const fn unclaimed_reward(&self) -> Amount {
        self.unclaimed_reward
    }
    pub const fn is_settled(&self, current_time: Seconds) -> bool {
        current_time >= self.end
    }
}

impl<N: FixedNumber> TryFrom<EpochSnapshot<N>> for Epoch<N> {
    type Error = RewardClearingError;

    fn try_from(snapshot: EpochSnapshot<N>) -> Result<Self> {
        let terms = EpochTerms::try_new(
            snapshot.duration,
            snapshot.ramp_bonus_slope,
            snapshot.exit_penalty_exponent,
        )?;
        let schedule = EpochSchedule::try_new(snapshot.master_start, snapshot.duration)?;
        let window = schedule.window(snapshot.epoch_counter)?;
        if snapshot.start != window.start {
            return Err(RewardClearingError::EpochScheduleMismatch);
        }
        crate::amount::from_amount::<N>(snapshot.unclaimed_reward)?;
        validate_principal_share_bounds(
            snapshot.principal,
            snapshot.unclaimed_shares,
            terms.maximum_entry_weight()?,
            RewardClearingError::InvalidEpochAccounting,
        )?;
        Ok(Self {
            master_start: snapshot.master_start,
            epoch_counter: snapshot.epoch_counter,
            start: snapshot.start,
            end: window.end,
            terms,
            principal: snapshot.principal,
            unclaimed_shares: snapshot.unclaimed_shares,
            unclaimed_reward: snapshot.unclaimed_reward,
        })
    }
}

impl<N: FixedNumber> From<Epoch<N>> for EpochSnapshot<N> {
    fn from(epoch: Epoch<N>) -> Self {
        Self {
            master_start: epoch.master_start,
            epoch_counter: epoch.epoch_counter,
            start: epoch.start,
            duration: epoch.terms.duration().get(),
            ramp_bonus_slope: epoch.terms.ramp_bonus_slope().get().get(),
            exit_penalty_exponent: epoch.terms.exit_penalty_exponent().get(),
            principal: epoch.principal,
            unclaimed_shares: epoch.unclaimed_shares,
            unclaimed_reward: epoch.unclaimed_reward,
        }
    }
}

/// Numeric inputs for validating frozen settlement totals and remaining claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinalizedEpochSnapshot<N: FixedNumber> {
    pub master_start: Seconds,
    pub epoch_counter: EpochCounter,
    pub start: Seconds,
    pub duration: Seconds,
    pub ramp_bonus_slope: N,
    pub exit_penalty_exponent: u32,
    pub settlement_shares: NonNeg<N>,
    pub settlement_reward: Amount,
    pub unclaimed_shares: NonNeg<N>,
    pub unclaimed_reward: Amount,
}

/// Immutable settlement ratio and the portion not yet claimed.
///
/// Principal is absent because settlement has already forfeited it. Every claim
/// uses the original totals, so claim order cannot redirect rounding residue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinalizedEpoch<N: FixedNumber> {
    pub(crate) master_start: Seconds,
    pub(crate) epoch_counter: EpochCounter,
    pub(crate) start: Seconds,
    pub(crate) end: Seconds,
    pub(crate) terms: EpochTerms<N>,
    pub(crate) settlement_shares: NonNeg<N>,
    pub(crate) settlement_reward: Amount,
    pub(crate) unclaimed_shares: NonNeg<N>,
    pub(crate) unclaimed_reward: Amount,
}

impl<N: FixedNumber> FinalizedEpoch<N> {
    pub const fn master_start(&self) -> Seconds {
        self.master_start
    }
    pub const fn epoch_counter(&self) -> EpochCounter {
        self.epoch_counter
    }
    pub const fn start(&self) -> Seconds {
        self.start
    }
    pub const fn end(&self) -> Seconds {
        self.end
    }
    pub const fn duration(&self) -> EpochDuration {
        self.terms.duration()
    }
    pub const fn terms(&self) -> EpochTerms<N> {
        self.terms
    }
    pub const fn settlement_shares(&self) -> NonNeg<N> {
        self.settlement_shares
    }
    pub const fn settlement_reward(&self) -> Amount {
        self.settlement_reward
    }
    pub const fn unclaimed_shares(&self) -> NonNeg<N> {
        self.unclaimed_shares
    }
    pub const fn unclaimed_reward(&self) -> Amount {
        self.unclaimed_reward
    }
    pub fn claims_complete(&self) -> bool {
        self.unclaimed_shares == NonNeg::zero()
    }
    pub const fn is_settled(&self, current_time: Seconds) -> bool {
        current_time >= self.end
    }

    /// Reward left after all eligible shares have claimed. A zero-share
    /// settlement makes its entire reward available immediately.
    pub fn remaining_reward(&self) -> Result<Amount> {
        if !self.claims_complete() {
            return Err(RewardClearingError::EpochClaimsOutstanding);
        }
        Ok(self.unclaimed_reward)
    }
}

impl<N: FixedNumber> TryFrom<FinalizedEpochSnapshot<N>> for FinalizedEpoch<N> {
    type Error = RewardClearingError;

    fn try_from(snapshot: FinalizedEpochSnapshot<N>) -> Result<Self> {
        let terms = EpochTerms::try_new(
            snapshot.duration,
            snapshot.ramp_bonus_slope,
            snapshot.exit_penalty_exponent,
        )?;
        let schedule = EpochSchedule::try_new(snapshot.master_start, snapshot.duration)?;
        let window = schedule.window(snapshot.epoch_counter)?;
        if snapshot.start != window.start {
            return Err(RewardClearingError::EpochScheduleMismatch);
        }
        crate::amount::from_amount::<N>(snapshot.settlement_reward)?;
        if snapshot.unclaimed_shares > snapshot.settlement_shares
            || snapshot.unclaimed_reward > snapshot.settlement_reward
        {
            return Err(RewardClearingError::FinalizedAccountingExceedsSettlement);
        }
        Ok(Self {
            master_start: snapshot.master_start,
            epoch_counter: snapshot.epoch_counter,
            start: snapshot.start,
            end: window.end,
            terms,
            settlement_shares: snapshot.settlement_shares,
            settlement_reward: snapshot.settlement_reward,
            unclaimed_shares: snapshot.unclaimed_shares,
            unclaimed_reward: snapshot.unclaimed_reward,
        })
    }
}

impl<N: FixedNumber> From<FinalizedEpoch<N>> for FinalizedEpochSnapshot<N> {
    fn from(epoch: FinalizedEpoch<N>) -> Self {
        Self {
            master_start: epoch.master_start,
            epoch_counter: epoch.epoch_counter,
            start: epoch.start,
            duration: epoch.terms.duration().get(),
            ramp_bonus_slope: epoch.terms.ramp_bonus_slope().get().get(),
            exit_penalty_exponent: epoch.terms.exit_penalty_exponent().get(),
            settlement_shares: epoch.settlement_shares,
            settlement_reward: epoch.settlement_reward,
            unclaimed_shares: epoch.unclaimed_shares,
            unclaimed_reward: epoch.unclaimed_reward,
        }
    }
}

/// Numeric totals for one participant's surviving lots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionSnapshot<N: FixedNumber> {
    pub principal: Amount,
    pub shares: NonNeg<N>,
    pub claimed: bool,
}

/// Aggregate participant principal and shares within one auction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position<N: FixedNumber> {
    pub(crate) principal: Amount,
    pub(crate) shares: NonNeg<N>,
    pub(crate) claimed: bool,
}

impl<N: FixedNumber> Position<N> {
    pub fn new() -> Self {
        Self {
            principal: 0,
            shares: NonNeg::zero(),
            claimed: false,
        }
    }
    pub const fn principal(&self) -> Amount {
        self.principal
    }
    pub const fn shares(&self) -> NonNeg<N> {
        self.shares
    }
    pub const fn is_claimed(&self) -> bool {
        self.claimed
    }
}

impl<N: FixedNumber> Default for Position<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N: FixedNumber> TryFrom<PositionSnapshot<N>> for Position<N> {
    type Error = RewardClearingError;
    fn try_from(snapshot: PositionSnapshot<N>) -> Result<Self> {
        validate_principal_share_bounds(
            snapshot.principal,
            snapshot.shares,
            maximum_entry_weight()?,
            RewardClearingError::InvalidPositionAccounting,
        )?;
        Ok(Self {
            principal: snapshot.principal,
            shares: snapshot.shares,
            claimed: snapshot.claimed,
        })
    }
}

impl<N: FixedNumber> From<Position<N>> for PositionSnapshot<N> {
    fn from(position: Position<N>) -> Self {
        Self {
            principal: position.principal,
            shares: position.shares,
            claimed: position.claimed,
        }
    }
}

/// Numeric terms and remaining quantities for one deposit lot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LotSnapshot<N: FixedNumber> {
    pub principal: Amount,
    pub shares: NonNeg<N>,
    pub entry_weight: NonNeg<N>,
}

/// A deposit's remaining principal and shares under its immutable entry weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lot<N: FixedNumber> {
    pub(crate) principal: Amount,
    pub(crate) shares: NonNeg<N>,
    pub(crate) entry_weight: NonNeg<N>,
}

impl<N: FixedNumber> Lot<N> {
    pub const fn principal(&self) -> Amount {
        self.principal
    }
    pub const fn shares(&self) -> NonNeg<N> {
        self.shares
    }
    pub const fn entry_weight(&self) -> NonNeg<N> {
        self.entry_weight
    }
}

impl<N: FixedNumber> TryFrom<LotSnapshot<N>> for Lot<N> {
    type Error = RewardClearingError;
    fn try_from(snapshot: LotSnapshot<N>) -> Result<Self> {
        let one = NonNeg::one();
        let maximum = maximum_entry_weight()?;
        if snapshot.entry_weight < one || snapshot.entry_weight > maximum {
            return Err(RewardClearingError::InvalidLotAccounting);
        }
        validate_principal_share_bounds(
            snapshot.principal,
            snapshot.shares,
            snapshot.entry_weight,
            RewardClearingError::InvalidLotAccounting,
        )?;
        Ok(Self {
            principal: snapshot.principal,
            shares: snapshot.shares,
            entry_weight: snapshot.entry_weight,
        })
    }
}

impl<N: FixedNumber> From<Lot<N>> for LotSnapshot<N> {
    fn from(lot: Lot<N>) -> Self {
        Self {
            principal: lot.principal,
            shares: lot.shares,
            entry_weight: lot.entry_weight,
        }
    }
}

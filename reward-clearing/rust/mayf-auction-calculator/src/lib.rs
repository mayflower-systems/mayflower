#![no_std]
#![forbid(unsafe_code)]

//! Calculate the weight and exit cost of commitments in a fixed time window.
//!
//! Callers supply execution time in seconds and principal in whole units.
//! Opening time is inclusive and the deadline is exclusive. Use one
//! withdrawal calculation for the entire requested amount.
//!
//! ```
//! use mayf_auction_calculator::{AuctionTerms, AuctionWindow, TimeWindow};
//! use mayf_fixed_drivers::U64F64;
//!
//! let terms = AuctionTerms::new(U64F64::ONE, 2)?;
//! let window = AuctionWindow::new(TimeWindow::new(0, 3_600), terms);
//!
//! let opening_shares = window.entry(0)?.shares(100)?;
//! assert_eq!(opening_shares.get(), U64F64::from_u64(200));
//!
//! let withdrawal = window.split_withdrawal(1_800, 40)?;
//! assert_eq!((withdrawal.returned, withdrawal.forfeited), (30, 10));
//! # Ok::<(), mayf_auction_calculator::AuctionError>(())
//! ```

use core::num::NonZeroU64;

use mayf_fixed_number::{DirectedRounding, FixedNumber, IntegralRounding, NonNeg};

/// Failures in term validation, window checks, or amount calculation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuctionError {
    InvalidDuration,
    InvalidRampBonusSlope,
    InvalidExitPenaltyExponent,
    WindowNotStarted,
    WindowEnded,
    WindowEndOverflow,
    EntryWeightOverflow,
    ExitPenaltyOverflow,
    SharesOverflow,
    DepositTooSmall,
    RefundOverflow,
    ForfeitureUnderflow,
}

pub type Result<T> = core::result::Result<T, AuctionError>;

/// Validated entry bonus and exit penalty parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuctionTerms<N: FixedNumber> {
    ramp_bonus_slope: NonNeg<N>,
    exit_penalty_exponent: u32,
}

impl<N: FixedNumber> AuctionTerms<N> {
    pub const MAX_RAMP_BONUS_SLOPE: i32 = 2;
    pub const MIN_EXIT_PENALTY_EXPONENT: u32 = 1;
    pub const MAX_EXIT_PENALTY_EXPONENT: u32 = 4;

    /// Accepts a slope in `[0, 2]` and an integer exponent in `[1, 4]`.
    pub fn new(ramp_bonus_slope: N, exit_penalty_exponent: u32) -> Result<Self> {
        let max_slope = N::from_i32(Self::MAX_RAMP_BONUS_SLOPE)
            .map_err(|_| AuctionError::InvalidRampBonusSlope)?;
        if ramp_bonus_slope > max_slope {
            return Err(AuctionError::InvalidRampBonusSlope);
        }
        let ramp_bonus_slope =
            NonNeg::new(ramp_bonus_slope).map_err(|_| AuctionError::InvalidRampBonusSlope)?;

        let exponents = Self::MIN_EXIT_PENALTY_EXPONENT..=Self::MAX_EXIT_PENALTY_EXPONENT;
        if !exponents.contains(&exit_penalty_exponent) {
            return Err(AuctionError::InvalidExitPenaltyExponent);
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

/// A start time and duration in seconds.
#[derive(Clone, Copy)]
pub struct TimeWindow {
    start: u64,
    duration: u64,
}

impl TimeWindow {
    /// Stores the schedule. Time queries check the deadline for overflow.
    /// A zero-duration window has no live instants.
    pub fn new(start: u64, duration: u64) -> Self {
        Self { start, duration }
    }

    /// The exclusive deadline, with checked addition.
    pub fn end(&self) -> Result<u64> {
        self.start
            .checked_add(self.duration)
            .ok_or(AuctionError::WindowEndOverflow)
    }

    /// Time left in the live interval. The opening instant is inclusive.
    pub fn time_remaining(&self, now: u64) -> Result<NonZeroU64> {
        if now < self.start {
            return Err(AuctionError::WindowNotStarted);
        }

        self.end()?
            .checked_sub(now)
            .and_then(NonZeroU64::new)
            .ok_or(AuctionError::WindowEnded)
    }

    /// Seconds since the start, while the window is live.
    pub fn elapsed(&self, now: u64) -> Result<u64> {
        Ok(self.duration - self.time_remaining(now)?.get())
    }

    fn duration(&self) -> Result<NonZeroU64> {
        NonZeroU64::new(self.duration).ok_or(AuctionError::InvalidDuration)
    }
}

/// A single time window with fixed curve terms.
pub struct AuctionWindow<N: FixedNumber> {
    time: TimeWindow,
    terms: AuctionTerms<N>,
}

/// The fixed-point weight quoted at entry, which can issue shares for an amount.
pub struct EntryWeight<N: FixedNumber>(NonNeg<N>);

/// The whole units returned and forfeited from one withdrawal.
pub struct Withdrawal {
    pub returned: u64,
    pub forfeited: u64,
}

impl<N: FixedNumber> AuctionWindow<N> {
    pub fn new(time: TimeWindow, terms: AuctionTerms<N>) -> Self {
        Self { time, terms }
    }

    /// Quote `1 + slope * remaining / duration` during the live window.
    /// The remaining fraction and bonus round down to the fixed-point grid.
    pub fn entry(&self, now: u64) -> Result<EntryWeight<N>> {
        let remaining = self.time.time_remaining(now)?;
        let duration = self.time.duration()?;

        let remaining_fraction = N::checked_from_ratio_round(
            remaining.get().into(),
            duration.into(),
            DirectedRounding::Floor,
        )
        .and_then(NonNeg::new)
        .map_err(|_| AuctionError::EntryWeightOverflow)?;
        let bonus = self
            .terms
            .ramp_bonus_slope()
            .checked_mul_floor(remaining_fraction)
            .map_err(|_| AuctionError::EntryWeightOverflow)?;

        NonNeg::one()
            .checked_add(bonus)
            .map(EntryWeight::new)
            .map_err(|_| AuctionError::EntryWeightOverflow)
    }

    /// Quote `(elapsed / duration)^exponent` during the live window.
    /// The ratio and each power multiplication round up. The result is capped at one.
    pub fn exit_penalty(&self, now: u64) -> Result<NonNeg<N>> {
        let elapsed = self.time.elapsed(now)?;
        let duration = self.time.duration()?;
        let penalty =
            N::checked_from_ratio_round(elapsed.into(), duration.into(), DirectedRounding::Ceil)
                .and_then(|progress| progress.checked_pow_ceil(self.terms.exit_penalty_exponent()))
                .and_then(NonNeg::new)
                .map_err(|_| AuctionError::ExitPenaltyOverflow)?;

        Ok(penalty.min(NonNeg::one()))
    }
}

impl<N: IntegralRounding> AuctionWindow<N> {
    /// Round the refund down to whole units and forfeit the difference.
    pub fn split_withdrawal(&self, now: u64, principal: u64) -> Result<Withdrawal> {
        let penalty = self.exit_penalty(now)?;
        let returned = NonNeg::<N>::one()
            .checked_sub(penalty)
            .and_then(|kept| NonNeg::<N>::from_u64(principal)?.checked_mul_floor(kept))
            .and_then(|kept| kept.get().checked_floor_to_u64())
            .map_err(|_| AuctionError::RefundOverflow)?;
        let forfeited = principal
            .checked_sub(returned)
            .ok_or(AuctionError::ForfeitureUnderflow)?;

        Ok(Withdrawal {
            returned,
            forfeited,
        })
    }
}

impl<N: FixedNumber> EntryWeight<N> {
    /// Wrap an already computed entry weight.
    pub fn new(weight: NonNeg<N>) -> Self {
        Self(weight)
    }

    pub fn get(self) -> NonNeg<N> {
        self.0
    }

    /// Multiply principal by the quoted weight and round down to the grid.
    /// Returns an error when the result would issue zero shares.
    pub fn shares(self, principal: u64) -> Result<NonNeg<N>> {
        let shares = NonNeg::<N>::from_u64(principal)
            .and_then(|principal| principal.checked_mul_floor(self.0))
            .map_err(|_| AuctionError::SharesOverflow)?;
        if shares == NonNeg::zero() {
            return Err(AuctionError::DepositTooSmall);
        }
        Ok(shares)
    }
}

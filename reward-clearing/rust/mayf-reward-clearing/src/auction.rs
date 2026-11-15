//! Calculate the weight and exit cost of commitments in one fixed window.
//!
//! The caller supplies the execution time and amount. A withdrawal split
//! applies to the entire amount in one request, even when it spans lots.

use core::num::NonZeroU64;

use mayf_fixed_number::{DirectedRounding, FixedNumber, IntegralRounding, NonNeg};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AuctionError {
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

type Result<T> = core::result::Result<T, AuctionError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AuctionTerms<N: FixedNumber> {
    ramp_bonus_slope: NonNeg<N>,
    exit_penalty_exponent: u32,
}

impl<N: FixedNumber> AuctionTerms<N> {
    pub(crate) const MAX_RAMP_BONUS_SLOPE: i32 = 2;
    pub(crate) const MIN_EXIT_PENALTY_EXPONENT: u32 = 1;
    pub(crate) const MAX_EXIT_PENALTY_EXPONENT: u32 = 4;

    pub(crate) fn new(ramp_bonus_slope: N, exit_penalty_exponent: u32) -> Result<Self> {
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

    pub(crate) fn ramp_bonus_slope(&self) -> NonNeg<N> {
        self.ramp_bonus_slope
    }

    pub(crate) fn exit_penalty_exponent(&self) -> u32 {
        self.exit_penalty_exponent
    }
}

#[derive(Clone, Copy)]
pub(crate) struct TimeWindow {
    start: u64,
    duration: u64,
}

impl TimeWindow {
    pub(crate) fn new(start: u64, duration: u64) -> Self {
        Self { start, duration }
    }

    pub(crate) fn end(&self) -> Result<u64> {
        self.start
            .checked_add(self.duration)
            .ok_or(AuctionError::WindowEndOverflow)
    }

    /// Time left in the live interval. The opening instant is inclusive.
    pub(crate) fn time_remaining(&self, now: u64) -> Result<NonZeroU64> {
        if now < self.start {
            return Err(AuctionError::WindowNotStarted);
        }

        self.end()?
            .checked_sub(now)
            .and_then(NonZeroU64::new)
            .ok_or(AuctionError::WindowEnded)
    }

    pub(crate) fn elapsed(&self, now: u64) -> Result<u64> {
        Ok(self.duration - self.time_remaining(now)?.get())
    }

    fn duration(&self) -> Result<NonZeroU64> {
        NonZeroU64::new(self.duration).ok_or(AuctionError::InvalidDuration)
    }
}

pub(crate) struct AuctionWindow<N: FixedNumber> {
    time: TimeWindow,
    terms: AuctionTerms<N>,
}

/// The fixed-point weight quoted at entry, which can issue shares for an amount.
pub(crate) struct EntryWeight<N: FixedNumber>(NonNeg<N>);

pub(crate) struct Withdrawal {
    pub(crate) returned: u64,
    pub(crate) forfeited: u64,
}

impl<N: FixedNumber> AuctionWindow<N> {
    pub(crate) fn new(time: TimeWindow, terms: AuctionTerms<N>) -> Self {
        Self { time, terms }
    }

    pub(crate) fn entry(&self, now: u64) -> Result<EntryWeight<N>> {
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

    pub(crate) fn exit_penalty(&self, now: u64) -> Result<NonNeg<N>> {
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
    pub(crate) fn split_withdrawal(&self, now: u64, principal: u64) -> Result<Withdrawal> {
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
    pub(crate) fn new(weight: NonNeg<N>) -> Self {
        Self(weight)
    }

    pub(crate) fn get(self) -> NonNeg<N> {
        self.0
    }

    pub(crate) fn shares(self, principal: u64) -> Result<NonNeg<N>> {
        let shares = NonNeg::<N>::from_u64(principal)
            .and_then(|principal| principal.checked_mul_floor(self.0))
            .map_err(|_| AuctionError::SharesOverflow)?;
        if shares == NonNeg::zero() {
            return Err(AuctionError::DepositTooSmall);
        }
        Ok(shares)
    }
}

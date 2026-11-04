//! Exact persistent geometry and supply accounting, independent of the engine.

use mayf_fixed_number::{FixedNumber, NonNeg, Pos};

use crate::error::arithmetic;
use crate::{Error, Result};

/// A floor and ramp on the permanent characteristic-progress axis.
///
/// Ramp start is exactly `ramp_end - ramp_width`. The main segment is derived
/// from the ramp endpoint when compiling a calculator; it has no independent
/// persistent anchor or vertical offset. Zero ramp width is valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segmentation<N: FixedNumber> {
    floor: Pos<N>,
    ramp_end: NonNeg<N>,
    ramp_width: NonNeg<N>,
}

impl<N: FixedNumber> Segmentation<N> {
    pub fn new(floor: Pos<N>, ramp_end: NonNeg<N>, ramp_width: NonNeg<N>) -> Result<Self> {
        if ramp_width > ramp_end {
            return Err(Error::RampWidthExceedsEnd);
        }
        arithmetic(ramp_end.checked_sub(ramp_width))?;
        Ok(Self {
            floor,
            ramp_end,
            ramp_width,
        })
    }

    /// Start with main `floor + f(x)` and no ramp or flat tail.
    pub fn launch(floor: Pos<N>) -> Self {
        Self {
            floor,
            ramp_end: NonNeg::zero(),
            ramp_width: NonNeg::zero(),
        }
    }

    pub const fn floor(&self) -> Pos<N> {
        self.floor
    }

    pub const fn ramp_end(&self) -> NonNeg<N> {
        self.ramp_end
    }

    pub const fn ramp_width(&self) -> NonNeg<N> {
        self.ramp_width
    }

    pub fn ramp_start(&self) -> Result<NonNeg<N>> {
        arithmetic(self.ramp_end.checked_sub(self.ramp_width))
    }

    /// Retain the old prefix through `progress` and attach main at that point.
    ///
    /// This changes only exact geometry. It needs no characteristic evaluation,
    /// inverse, or rounded price. A later compilation derives the new main.
    pub fn contract_to(&self, progress: NonNeg<N>) -> Result<Self> {
        if progress >= self.ramp_end {
            return Ok(*self);
        }
        let ramp_start = self.ramp_start()?.min(progress);
        Self::new(
            self.floor,
            progress,
            arithmetic(progress.checked_sub(ramp_start))?,
        )
    }
}

/// Progress along the characteristic and independently circulating supply.
///
/// Floor issuance and redemption change circulation without moving progress.
/// Pair accounting with segmentation through [`MarketState::new`] to check the
/// relationships between these counters and the schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Accounting<N: FixedNumber> {
    progress: NonNeg<N>,
    circulating_supply: NonNeg<N>,
}

impl<N: FixedNumber> Accounting<N> {
    pub const fn new(progress: NonNeg<N>, circulating_supply: NonNeg<N>) -> Self {
        Self {
            progress,
            circulating_supply,
        }
    }

    pub const fn progress(&self) -> NonNeg<N> {
        self.progress
    }

    pub const fn circulating_supply(&self) -> NonNeg<N> {
        self.circulating_supply
    }
}

/// Geometry and accounting whose combined supply invariants have been checked.
///
/// Progress is at least ramp end, so ordinary issuance begins on main.
/// Circulation covers `progress - ramp_start`; any remainder is the physical
/// floor tail. Engine configuration and reserve custody belong to the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarketState<N: FixedNumber> {
    segmentation: Segmentation<N>,
    accounting: Accounting<N>,
}

impl<N: FixedNumber> MarketState<N> {
    pub fn new(segmentation: Segmentation<N>, accounting: Accounting<N>) -> Result<Self> {
        if accounting.progress < segmentation.ramp_end {
            return Err(Error::RampEndBeyondProgress);
        }
        let nonfloor_supply =
            arithmetic(accounting.progress.checked_sub(segmentation.ramp_start()?))?;
        if accounting.circulating_supply < nonfloor_supply {
            return Err(Error::InsufficientCirculation);
        }
        Ok(Self {
            segmentation,
            accounting,
        })
    }

    pub fn launch(floor: Pos<N>) -> Self {
        Self {
            segmentation: Segmentation::launch(floor),
            accounting: Accounting::new(NonNeg::zero(), NonNeg::zero()),
        }
    }

    pub const fn segmentation(&self) -> Segmentation<N> {
        self.segmentation
    }

    pub const fn accounting(&self) -> Accounting<N> {
        self.accounting
    }

    /// Available physical floor tail, evaluated without a signed supply offset.
    pub fn floor_tail(&self) -> Result<NonNeg<N>> {
        let nonfloor_supply = arithmetic(
            self.accounting
                .progress
                .checked_sub(self.segmentation.ramp_start()?),
        )?;
        arithmetic(
            self.accounting
                .circulating_supply
                .checked_sub(nonfloor_supply),
        )
    }
}

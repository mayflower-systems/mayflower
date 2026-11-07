//! Exact persistent geometry and supply accounting, independent of the engine.

use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::{Error, Result};

/// A floor and ramp on the permanent characteristic-progress axis.
///
/// Ramp start is exactly `ramp_end - ramp_width`. Main is `f(x) + main_offset`;
/// floor raises preserve that offset. Zero ramp width is valid. The constructor
/// checks geometry; engine-dependent floor joins are checked by the floor-raise
/// verifier. Restored state must originate from verified transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segmentation<N: FixedNumber> {
    floor: NonNeg<N>,
    ramp_end: NonNeg<N>,
    ramp_width: NonNeg<N>,
    main_offset: N,
}

impl<N: FixedNumber> Segmentation<N> {
    pub fn new(
        floor: NonNeg<N>,
        ramp_end: NonNeg<N>,
        ramp_width: NonNeg<N>,
        main_offset: N,
    ) -> Result<Self> {
        if ramp_width > ramp_end {
            return Err(Error::RampWidthExceedsEnd);
        }
        ramp_end
            .checked_sub(ramp_width)
            .map_err(|_| Error::ArithmeticFailure)?;
        Ok(Self {
            floor,
            ramp_end,
            ramp_width,
            main_offset,
        })
    }

    /// Start with main `floor + f(x)` and no ramp or flat tail.
    pub fn launch(floor: NonNeg<N>) -> Self {
        Self {
            floor,
            ramp_end: NonNeg::zero(),
            ramp_width: NonNeg::zero(),
            main_offset: floor.get(),
        }
    }

    pub const fn floor(&self) -> NonNeg<N> {
        self.floor
    }

    pub const fn ramp_end(&self) -> NonNeg<N> {
        self.ramp_end
    }

    pub const fn ramp_width(&self) -> NonNeg<N> {
        self.ramp_width
    }

    pub fn ramp_start(&self) -> Result<NonNeg<N>> {
        self.ramp_end
            .checked_sub(self.ramp_width)
            .map_err(|_| Error::ArithmeticFailure)
    }

    pub const fn main_offset(&self) -> N {
        self.main_offset
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
        let nonfloor_supply = accounting
            .progress
            .checked_sub(segmentation.ramp_start()?)
            .map_err(|_| Error::ArithmeticFailure)?;
        if accounting.circulating_supply < nonfloor_supply {
            return Err(Error::InsufficientCirculation);
        }
        Ok(Self {
            segmentation,
            accounting,
        })
    }

    pub fn launch(floor: NonNeg<N>) -> Self {
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
        let nonfloor_supply = self
            .accounting
            .progress
            .checked_sub(self.segmentation.ramp_start()?)
            .map_err(|_| Error::ArithmeticFailure)?;
        self.accounting
            .circulating_supply
            .checked_sub(nonfloor_supply)
            .map_err(|_| Error::ArithmeticFailure)
    }
}

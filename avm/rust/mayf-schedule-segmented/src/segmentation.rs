//! Persistable geometry and main placement on the net-issued supply axis.

use mayf_fixed_number::{AboveOne, Bounds, FixedNumber, NonNeg};

use crate::error::arithmetic;
use crate::{ScheduleError, ScheduleResult};

/// A certified main-curve value at a characteristic-input coordinate.
///
/// Main is `rate + f(x) - f(curve_x)`. The producer certifies the rate's
/// enclosure; constructing ordered bounds alone does not prove its provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainReference<N: FixedNumber> {
    curve_x: NonNeg<N>,
    rate: Bounds<NonNeg<N>>,
}

impl<N: FixedNumber> MainReference<N> {
    pub const fn new(curve_x: NonNeg<N>, rate: Bounds<NonNeg<N>>) -> Self {
        Self { curve_x, rate }
    }

    pub fn exact(curve_x: NonNeg<N>, rate: NonNeg<N>) -> Self {
        Self::new(curve_x, Bounds::exact(rate))
    }

    pub const fn curve_x(&self) -> NonNeg<N> {
        self.curve_x
    }

    pub const fn rate(&self) -> Bounds<NonNeg<N>> {
        self.rate
    }
}

/// Immutable floor/ramp/main state in net-issued coordinates.
///
/// Ramp start is exactly `ramp_end - ramp_width`. Main placement is independent
/// of the floor and ramp, so changing segmentation need not move main.
/// Construction checks geometry and component domains; continuity and economic
/// acceptance of a candidate belong to higher layers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segmentation<N: FixedNumber> {
    floor: NonNeg<N>,
    ramp_end: NonNeg<N>,
    ramp_width: NonNeg<N>,
    ramp_scalar: AboveOne<N>,
    main_reference: MainReference<N>,
}

impl<N: FixedNumber> Segmentation<N> {
    pub fn new(
        floor: NonNeg<N>,
        ramp_end: NonNeg<N>,
        ramp_width: NonNeg<N>,
        ramp_scalar: AboveOne<N>,
        main_reference: MainReference<N>,
    ) -> ScheduleResult<Self> {
        if ramp_width > ramp_end {
            return Err(ScheduleError::RampWidthExceedsEnd);
        }
        if main_reference.curve_x > ramp_end {
            return Err(ScheduleError::MainReferenceAfterRampEnd);
        }
        if ramp_end == NonNeg::zero() && main_reference.rate != Bounds::exact(floor) {
            return Err(ScheduleError::InvalidOriginReference);
        }
        arithmetic(ramp_end.checked_sub(ramp_width))?;

        Ok(Self {
            floor,
            ramp_end,
            ramp_width,
            ramp_scalar,
            main_reference,
        })
    }

    /// A zero-width schedule whose main is `floor + f(x)` from coordinate zero.
    pub fn launch(floor: NonNeg<N>, ramp_scalar: AboveOne<N>) -> Self {
        Self {
            floor,
            ramp_end: NonNeg::zero(),
            ramp_width: NonNeg::zero(),
            ramp_scalar,
            main_reference: MainReference::exact(NonNeg::zero(), floor),
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

    pub fn ramp_start(&self) -> ScheduleResult<NonNeg<N>> {
        arithmetic(self.ramp_end.checked_sub(self.ramp_width))
    }

    pub const fn ramp_scalar(&self) -> AboveOne<N> {
        self.ramp_scalar
    }

    pub const fn main_reference(&self) -> MainReference<N> {
        self.main_reference
    }
}

//! Optional horizontal translation of a compiled segmented schedule.

use mayf_fixed_number::{Bounds, NonNeg, Pos, SignedFixed};
use mayf_monotone::MonotoneCalculator;

use crate::calculator::{add_bounds, constant_area};
use crate::error::arithmetic;
use crate::{ScheduleError, ScheduleResult, SegmentedScheduleCalculator};

/// A horizontal translation of a core calculator, evaluated in supply coordinates.
///
/// Positive shifts move both joins right and extend the floor. Negative shifts
/// consume the floor tail, up to its full width. Component calculators remain
/// unchanged; this wrapper translates only coordinates and handles the flat tail.
/// Token issuance, redemption, and supply accounting belong to callers.
pub struct ShiftedScheduleCalculator<N, K>
where
    N: SignedFixed,
    K: MonotoneCalculator<N>,
{
    inner: SegmentedScheduleCalculator<N, K>,
    horizontal_shift: N,
    ramp_start: NonNeg<N>,
    ramp_end: NonNeg<N>,
}

impl<N, K> ShiftedScheduleCalculator<N, K>
where
    N: SignedFixed,
    K: MonotoneCalculator<N>,
{
    pub fn new(
        inner: SegmentedScheduleCalculator<N, K>,
        horizontal_shift: N,
    ) -> ScheduleResult<Self> {
        let start = arithmetic(inner.ramp_start().get().checked_add(horizontal_shift))?;
        let ramp_start = NonNeg::new(start).map_err(|_| ScheduleError::NegativeRampStart)?;
        let end = arithmetic(
            inner
                .segmentation()
                .ramp_end()
                .get()
                .checked_add(horizontal_shift),
        )?;
        let ramp_end = NonNeg::new(end).map_err(|_| ScheduleError::NegativeRampStart)?;

        Ok(Self {
            inner,
            horizontal_shift,
            ramp_start,
            ramp_end,
        })
    }

    pub const fn inner(&self) -> &SegmentedScheduleCalculator<N, K> {
        &self.inner
    }

    pub fn into_inner(self) -> SegmentedScheduleCalculator<N, K> {
        self.inner
    }

    pub const fn horizontal_shift(&self) -> N {
        self.horizontal_shift
    }

    /// Exact ramp start in shifted supply coordinates; also the available tail.
    pub const fn ramp_start(&self) -> NonNeg<N> {
        self.ramp_start
    }

    pub const fn ramp_end(&self) -> NonNeg<N> {
        self.ramp_end
    }

    /// Translate by a signed displacement, reusing the compiled components.
    pub fn translate(self, displacement: N) -> ScheduleResult<Self> {
        let shift = arithmetic(self.horizontal_shift.checked_add(displacement))?;
        Self::new(self.inner, shift)
    }

    pub fn y(&self, x: NonNeg<N>) -> ScheduleResult<Bounds<NonNeg<N>>> {
        if x < self.ramp_start {
            return Ok(Bounds::exact(self.inner.segmentation().floor()));
        }
        self.inner.y(self.unshift(x)?)
    }

    pub fn integrate(&self, lo: NonNeg<N>, width: Pos<N>) -> ScheduleResult<Bounds<NonNeg<N>>> {
        let hi = arithmetic(lo.checked_add(width))?;
        let mut cursor = lo;
        let mut total = Bounds::exact(NonNeg::zero());

        // The translated input can be negative within a right-shifted floor.
        // Integrate that tail directly instead of evaluating the characteristic.
        if cursor < self.ramp_start {
            let end = hi.min(self.ramp_start);
            total = constant_area(
                self.inner.segmentation().floor(),
                arithmetic(end.checked_sub(cursor))?,
            )?;
            cursor = end;
        }
        if cursor < hi {
            let remaining = arithmetic(hi.checked_sub(cursor))?;
            let width = arithmetic(Pos::try_from(remaining))?;
            total = add_bounds(total, self.inner.integrate(self.unshift(cursor)?, width)?)?;
        }

        Ok(total)
    }

    fn unshift(&self, x: NonNeg<N>) -> ScheduleResult<NonNeg<N>> {
        let input = arithmetic(x.get().checked_sub(self.horizontal_shift))?;
        NonNeg::new(input).map_err(|_| ScheduleError::CharacteristicOutOfDomain)
    }
}

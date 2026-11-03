//! Affine component compilation and integration on the net-issued axis.

use mayf_characteristic::{AffineAnchor, AffineTransform, Characteristic};
use mayf_fixed_number::{Bounds, FixedNumber, NonNeg, Pos, SignedFixed};
use mayf_monotone::MonotoneCalculator;

use crate::error::arithmetic;
use crate::{MainReference, ScheduleResult, Segmentation};

/// Compile an immutable segmentation into disposable affine component views.
///
/// Compilation derives views without evaluating endpoints or checking joins.
/// Component arithmetic and domain errors are reported when queried.
pub fn compile_segmented_schedule<N, C>(
    characteristic: &C,
    segmentation: Segmentation<N>,
) -> ScheduleResult<SegmentedScheduleCalculator<N, C::Calculator>>
where
    N: SignedFixed,
    C: Characteristic<N>,
{
    let ramp_start = segmentation.ramp_start()?;
    let reference = segmentation.main_reference();
    let main = characteristic.calculator(AffineTransform::new(
        Pos::one(),
        N::ZERO,
        NonNeg::one(),
        AffineAnchor::new(reference.curve_x(), reference.rate()),
    ));
    let ramp = if segmentation.ramp_width() == NonNeg::zero() {
        None
    } else {
        let ramp = characteristic.calculator(AffineTransform::new(
            Pos::one(),
            N::ZERO,
            NonNeg::from(segmentation.ramp_scalar()),
            AffineAnchor::exact(ramp_start, segmentation.floor()),
        ));
        Some(ramp)
    };

    Ok(SegmentedScheduleCalculator {
        segmentation,
        ramp_start,
        ramp,
        main,
    })
}

/// A floor/ramp/main calculator in nonnegative net-issued coordinates.
///
/// Main placement is independent of the floor and ramp. The component views
/// are monotone, but an unverified candidate may have a discontinuous join.
/// This type therefore does not claim the composite `MonotoneCalculator`
/// contract before validation by a higher layer.
pub struct SegmentedScheduleCalculator<N, K>
where
    N: SignedFixed,
    K: MonotoneCalculator<N>,
{
    segmentation: Segmentation<N>,
    ramp_start: NonNeg<N>,
    ramp: Option<K>,
    main: K,
}

impl<N, K> SegmentedScheduleCalculator<N, K>
where
    N: SignedFixed,
    K: MonotoneCalculator<N>,
{
    pub const fn segmentation(&self) -> Segmentation<N> {
        self.segmentation
    }

    pub const fn ramp_start(&self) -> NonNeg<N> {
        self.ramp_start
    }

    /// Evaluate the selected component. Main owns the exact ramp-end point.
    pub fn y(&self, x: NonNeg<N>) -> ScheduleResult<Bounds<NonNeg<N>>> {
        if x < self.ramp_start {
            return Ok(Bounds::exact(self.segmentation.floor()));
        }
        if let Some(ramp) = &self.ramp
            && x < self.segmentation.ramp_end()
        {
            return ramp.y(x).map_err(Into::into);
        }
        self.main.y(x).map_err(Into::into)
    }

    /// Enclose the integral over `[lo, lo + width]`, splitting at exact joins.
    pub fn integrate(&self, lo: NonNeg<N>, width: Pos<N>) -> ScheduleResult<Bounds<NonNeg<N>>> {
        let hi = arithmetic(lo.checked_add(width))?;
        let mut cursor = lo;
        let mut total = Bounds::exact(NonNeg::zero());

        if cursor < self.ramp_start {
            let end = hi.min(self.ramp_start);
            total = constant_area(
                self.segmentation.floor(),
                arithmetic(end.checked_sub(cursor))?,
            )?;
            cursor = end;
        }
        if let Some(ramp) = &self.ramp
            && cursor < hi
            && cursor < self.segmentation.ramp_end()
        {
            let end = hi.min(self.segmentation.ramp_end());
            let area = ramp.integrate(cursor, arithmetic(end.checked_sub(cursor))?)?;
            total = add_bounds(total, area)?;
            cursor = end;
        }
        if cursor < hi {
            let area = self
                .main
                .integrate(cursor, arithmetic(hi.checked_sub(cursor))?)?;
            total = add_bounds(total, area)?;
        }

        Ok(total)
    }

    /// Contract the ramp to remaining net-issued supply without changing its
    /// retained prefix. The next issuance starts on unscaled main at that input.
    pub fn contract_to(&self, remaining_net_issued: NonNeg<N>) -> ScheduleResult<Segmentation<N>> {
        if remaining_net_issued >= self.segmentation.ramp_end() {
            return Ok(self.segmentation);
        }
        let (width, rate) = if remaining_net_issued >= self.ramp_start {
            (
                arithmetic(remaining_net_issued.checked_sub(self.ramp_start))?,
                self.y(remaining_net_issued)?,
            )
        } else {
            (NonNeg::zero(), Bounds::exact(self.segmentation.floor()))
        };

        Segmentation::new(
            self.segmentation.floor(),
            remaining_net_issued,
            width,
            self.segmentation.ramp_scalar(),
            MainReference::new(remaining_net_issued, rate),
        )
    }
}

pub(crate) fn constant_area<N: FixedNumber>(
    floor: NonNeg<N>,
    width: NonNeg<N>,
) -> ScheduleResult<Bounds<NonNeg<N>>> {
    arithmetic((floor.bounded() * width).bounds())
}

pub(crate) fn add_bounds<N: FixedNumber>(
    left: Bounds<NonNeg<N>>,
    right: Bounds<NonNeg<N>>,
) -> ScheduleResult<Bounds<NonNeg<N>>> {
    arithmetic((left.bounded() + right.bounded()).bounds())
}

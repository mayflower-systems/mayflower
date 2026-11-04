//! Compile the continuous schedule from exact geometry and a characteristic.

use mayf_characteristic::{AffineAnchor, AffineTransform, Characteristic};
use mayf_fixed_number::{AboveOne, Bounds, FixedNumber, NonNeg, Pos, SignedFixed};
use mayf_monotone::MonotoneCalculator;

use crate::error::arithmetic;
use crate::{Result, Segmentation};

/// Derive main's anchor from the ramp endpoint, preserving the entire enclosure.
///
/// The two components use the same neutral characteristic input `x`. Ramp has
/// gain `k` and anchor `(A, F)`; main has gain one and anchor `(E, ramp(E))`.
/// Both joins follow from these definitions, rather than independent witnesses.
pub fn compile_schedule<N, C>(
    characteristic: &C,
    ramp_scalar: AboveOne<N>,
    segmentation: Segmentation<N>,
) -> Result<ScheduleCalculator<N, C::Calculator>>
where
    N: SignedFixed,
    C: Characteristic<N>,
{
    let ramp_start = segmentation.ramp_start()?;
    let floor = NonNeg::from(segmentation.floor());
    let (ramp, main_rate) = if segmentation.ramp_width() == NonNeg::zero() {
        (None, Bounds::exact(floor))
    } else {
        let ramp = characteristic.calculator(AffineTransform::new(
            Pos::one(),
            N::ZERO,
            NonNeg::from(ramp_scalar),
            AffineAnchor::exact(ramp_start, floor),
        ));
        let main_rate = ramp.y(segmentation.ramp_end())?;
        (Some(ramp), main_rate)
    };
    let main = characteristic.calculator(AffineTransform::new(
        Pos::one(),
        N::ZERO,
        NonNeg::one(),
        AffineAnchor::new(segmentation.ramp_end(), main_rate),
    ));

    Ok(ScheduleCalculator {
        segmentation,
        ramp_start,
        ramp,
        main,
    })
}

/// Disposable floor/ramp/main calculators on the characteristic-progress axis.
///
/// The exact defining state is [`Segmentation`]. Derived endpoint bounds are
/// confined to these calculators and are never written back into segmentation.
pub struct ScheduleCalculator<N, K>
where
    N: SignedFixed,
    K: MonotoneCalculator<N>,
{
    segmentation: Segmentation<N>,
    ramp_start: NonNeg<N>,
    ramp: Option<K>,
    main: K,
}

impl<N, K> ScheduleCalculator<N, K>
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

    /// Enclose the price at a progress coordinate. Main owns the ramp endpoint.
    pub fn y(&self, x: NonNeg<N>) -> Result<Bounds<NonNeg<N>>> {
        if x < self.ramp_start {
            return Ok(Bounds::exact(NonNeg::from(self.segmentation.floor())));
        }
        if let Some(ramp) = &self.ramp
            && x < self.segmentation.ramp_end()
        {
            return ramp.y(x).map_err(Into::into);
        }
        self.main.y(x).map_err(Into::into)
    }

    /// Enclose the area over `[lo, lo + width]`, splitting only at exact joins.
    /// Zero width returns exact zero, matching the component calculator contract.
    pub fn integrate(&self, lo: NonNeg<N>, width: NonNeg<N>) -> Result<Bounds<NonNeg<N>>> {
        if width == NonNeg::zero() {
            return Ok(Bounds::exact(NonNeg::zero()));
        }
        let hi = arithmetic(lo.checked_add(width))?;
        let mut cursor = lo;
        let mut total = Bounds::exact(NonNeg::zero());

        if cursor < self.ramp_start {
            let end = hi.min(self.ramp_start);
            total = constant_area(
                NonNeg::from(self.segmentation.floor()),
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
}

pub(crate) fn constant_area<N: FixedNumber>(
    floor: NonNeg<N>,
    width: NonNeg<N>,
) -> Result<Bounds<NonNeg<N>>> {
    arithmetic((floor.bounded() * width).bounds())
}

pub(crate) fn add_bounds<N: FixedNumber>(
    left: Bounds<NonNeg<N>>,
    right: Bounds<NonNeg<N>>,
) -> Result<Bounds<NonNeg<N>>> {
    arithmetic((left.bounded() + right.bounded()).bounds())
}

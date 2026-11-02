//! Private just-in-time compilation and piecewise calculation machinery.
//!
//! This module observes valid immutable geometry. It does not construct or
//! transform persisted schedule state.

use mayf_characteristic::{AffineAnchor, AffineTransform, Characteristic};
use mayf_fixed_number::{Bounds, FixedNumber, MathResult, NonNeg, Pos, SignedFixed};
use mayf_monotone::{MonotoneCalculator, MonotoneCalculatorError, MonotoneCalculatorResult};

use crate::geometry::Segmentation;
use crate::schedule::{ScheduleError, ScheduleResult};

pub(crate) struct CompiledSegmentedSchedule<N, K>
where
    N: SignedFixed,
    K: MonotoneCalculator<N>,
{
    floor: ConstantCalculator<N>,
    ramp_start: NonNeg<N>,
    ramp_end: NonNeg<N>,
    ramp: K,
    main: K,
}

impl<N, K> CompiledSegmentedSchedule<N, K>
where
    N: SignedFixed,
    K: MonotoneCalculator<N>,
{
    pub(crate) fn compile<C>(
        characteristic: &C,
        segmentation: Segmentation<N>,
        horizontal_shift: NonNeg<N>,
    ) -> ScheduleResult<Self>
    where
        C: Characteristic<N, Calculator = K>,
    {
        let ramp_start = shifted_coordinate(segmentation.ramp_start(), horizontal_shift)?;
        let ramp_end = shifted_coordinate(segmentation.ramp_end(), horizontal_shift)?;
        let x_translation = horizontal_shift
            .get()
            .checked_neg()
            .map_err(|_| ScheduleError::ArithmeticFailure)?;
        let ramp_affine = AffineTransform::new(
            Pos::one(),
            x_translation,
            NonNeg::from(segmentation.ramp_scalar()),
            AffineAnchor::exact(ramp_start, segmentation.floor()),
        );
        let ramp = characteristic.calculator(ramp_affine);
        let ramp_end_y = calculator_result(ramp.y(ramp_end))?;

        let main_anchor = AffineAnchor::new(ramp_end, ramp_end_y);
        let main_affine =
            AffineTransform::new(Pos::one(), x_translation, NonNeg::one(), main_anchor);
        let main = characteristic.calculator(main_affine);
        calculator_result(main.y(ramp_end))?;

        Ok(Self {
            floor: ConstantCalculator {
                value: segmentation.floor(),
            },
            ramp_start,
            ramp_end,
            ramp,
            main,
        })
    }

    pub(crate) fn y(&self, x: NonNeg<N>) -> ScheduleResult<Bounds<NonNeg<N>>> {
        if x < self.ramp_start {
            calculator_result(self.floor.y(x))
        } else if x < self.ramp_end {
            calculator_result(self.ramp.y(x))
        } else {
            calculator_result(self.main.y(x))
        }
    }

    pub(crate) fn integrate(
        &self,
        lo: NonNeg<N>,
        width: NonNeg<N>,
    ) -> ScheduleResult<Bounds<NonNeg<N>>> {
        if width == NonNeg::zero() {
            return Ok(Bounds::exact(NonNeg::zero()));
        }
        let hi = arithmetic(lo.checked_add(width))?;

        let mut total = Bounds::exact(NonNeg::zero());
        let mut cursor = lo;

        if cursor < self.ramp_start {
            let end = hi.min(self.ramp_start);
            total = add_bounds(
                total,
                calculator_result(
                    self.floor
                        .integrate(cursor, positive_difference(cursor, end)?),
                )?,
            )?;
            cursor = end;
        }
        if cursor < hi && cursor < self.ramp_end {
            let end = hi.min(self.ramp_end);
            total = add_bounds(
                total,
                calculator_result(
                    self.ramp
                        .integrate(cursor, positive_difference(cursor, end)?),
                )?,
            )?;
            cursor = end;
        }
        if cursor < hi {
            total = add_bounds(
                total,
                calculator_result(
                    self.main
                        .integrate(cursor, positive_difference(cursor, hi)?),
                )?,
            )?;
        }

        Ok(total)
    }
}

#[derive(Clone, Copy, Debug)]
struct ConstantCalculator<N>
where
    N: FixedNumber,
{
    value: NonNeg<N>,
}

impl<N> MonotoneCalculator<N> for ConstantCalculator<N>
where
    N: FixedNumber,
{
    fn y(&self, _x: NonNeg<N>) -> MonotoneCalculatorResult<Bounds<NonNeg<N>>> {
        Ok(Bounds::exact(self.value))
    }

    fn integrate(
        &self,
        _lo: NonNeg<N>,
        width: NonNeg<N>,
    ) -> MonotoneCalculatorResult<Bounds<NonNeg<N>>> {
        (self.value.bounded() * width)
            .bounds()
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)
    }
}

fn shifted_coordinate<N>(
    coordinate: NonNeg<N>,
    horizontal_shift: NonNeg<N>,
) -> ScheduleResult<NonNeg<N>>
where
    N: FixedNumber,
{
    coordinate
        .checked_add(horizontal_shift)
        .map_err(|_| ScheduleError::ArithmeticFailure)
}

pub(crate) fn calculator_result<T>(result: MonotoneCalculatorResult<T>) -> ScheduleResult<T> {
    result.map_err(|error| match error {
        MonotoneCalculatorError::InputOutOfDomain => ScheduleError::CharacteristicOutOfDomain,
        MonotoneCalculatorError::InvalidInterval => ScheduleError::InvalidInterval,
        MonotoneCalculatorError::NegativeOutput => ScheduleError::NegativePrice,
        MonotoneCalculatorError::ArithmeticFailure => ScheduleError::ArithmeticFailure,
    })
}

fn arithmetic<T>(result: MathResult<T>) -> ScheduleResult<T> {
    result.map_err(|_| ScheduleError::ArithmeticFailure)
}

fn positive_difference<N>(lo: NonNeg<N>, hi: NonNeg<N>) -> ScheduleResult<NonNeg<N>>
where
    N: FixedNumber,
{
    if hi <= lo {
        return Err(ScheduleError::InvalidInterval);
    }

    arithmetic(hi.checked_sub(lo))
}

fn add_bounds<N>(
    left: Bounds<NonNeg<N>>,
    right: Bounds<NonNeg<N>>,
) -> ScheduleResult<Bounds<NonNeg<N>>>
where
    N: FixedNumber,
{
    arithmetic((left.bounded() + right.bounded()).bounds())
}

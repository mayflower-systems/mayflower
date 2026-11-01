use mayf_characteristic::{AffineTransform, Characteristic};
use mayf_characteristic_linear::{LinearParams, LinearParamsError};
use mayf_fixed_number::{Bounds, FixedNumber, MathResult, NonNeg, Pos, SignedFixed};
use mayf_monotone::{MonotoneCalculator, MonotoneCalculatorError, MonotoneCalculatorResult};

/// The origin-anchored characteristic `f(x) = slope * x`, with positive slope.
///
/// Only the neutral parameters are stored; [`Characteristic::calculator`]
/// supplies the disposable affine view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinearCharacteristic<N: FixedNumber> {
    params: LinearParams<N>,
}

impl<N: FixedNumber> LinearCharacteristic<N> {
    pub const fn new(slope: Pos<N>) -> Self {
        Self::from_params(LinearParams::new(slope))
    }

    pub fn try_new(slope: N) -> Result<Self, LinearParamsError> {
        LinearParams::try_new(slope).map(Self::from_params)
    }

    pub const fn from_params(params: LinearParams<N>) -> Self {
        Self { params }
    }

    pub const fn params(&self) -> &LinearParams<N> {
        &self.params
    }

    pub const fn slope(&self) -> Pos<N> {
        self.params.slope()
    }
}

/// An anchored affine view constructed by [`LinearCharacteristic`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinearCalculator<N: SignedFixed> {
    affine: AffineTransform<N>,
    slope: MathResult<LinearSlope<N>>,
}

/// Separate an integer slope from its fractional remainder. Multiplication by
/// the integer part is exact on every fixed grid. Integrating the remainder
/// after rounding the geometric span avoids amplifying that rounding by a
/// potentially huge whole slope. The remainder also carries slope uncertainty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinearSlope<N: FixedNumber> {
    bounds: Bounds<NonNeg<N>>,
    whole: NonNeg<N>,
    fraction: Bounds<NonNeg<N>>,
}

impl<N: FixedNumber> LinearSlope<N> {
    fn new(bounds: Bounds<NonNeg<N>>) -> MathResult<Self> {
        let mut whole = NonNeg::zero();
        let mut remainder = bounds.lower();
        if remainder >= NonNeg::one() {
            let two = Pos::from_i32(2)?;
            let mut step = NonNeg::one();
            while let Ok(next) = step.checked_add(step) {
                if next > remainder {
                    break;
                }
                step = next;
            }
            loop {
                if step <= remainder {
                    whole = whole.checked_add(step)?;
                    remainder = remainder.checked_sub(step)?;
                }
                if step == NonNeg::one() {
                    break;
                }
                // step is an integer power of two, so this division is exact.
                step = step.checked_div_floor(two)?;
            }
        }
        let fraction = Bounds::new(remainder, bounds.upper().checked_sub(whole)?)?;
        Ok(Self {
            bounds,
            whole,
            fraction,
        })
    }
}

impl<N: SignedFixed> Characteristic<N> for LinearCharacteristic<N> {
    type Calculator = LinearCalculator<N>;

    fn calculator(&self, affine: AffineTransform<N>) -> Self::Calculator {
        LinearCalculator {
            affine,
            slope: (self.params.slope().bounded() * affine.x_scale() * affine.y_scale())
                .bounds()
                .and_then(LinearSlope::new),
        }
    }
}

impl<N: SignedFixed> LinearCalculator<N> {
    /// Check the neutral coordinate even though translation cancels from the
    /// linear difference. Requiring its whole enclosure to be nonnegative
    /// keeps the affine view within the characteristic's supported domain.
    fn require_input_domain(&self, x: NonNeg<N>) -> MonotoneCalculatorResult<()> {
        if x < self.affine.anchor().x() {
            return Err(MonotoneCalculatorError::InputOutOfDomain);
        }
        let transformed = arithmetic((|| {
            let scaled = (x.bounded() * self.affine.x_scale()).bounds()?.into_raw();
            scaled.checked_add_exact(self.affine.x_translation())
        })())?;
        transformed
            .try_nonnegative()
            .map(|_| ())
            .map_err(|_| MonotoneCalculatorError::InputOutOfDomain)
    }
}

impl<N: SignedFixed> MonotoneCalculator<N> for LinearCalculator<N> {
    fn y(&self, x: NonNeg<N>) -> MonotoneCalculatorResult<Bounds<NonNeg<N>>> {
        self.require_input_domain(self.affine.anchor().x())?;
        self.require_input_domain(x)?;
        if x == self.affine.anchor().x() || self.affine.y_scale() == NonNeg::zero() {
            return Ok(self.affine.anchor().y());
        }

        // f(b*x+c) - f(b*anchor_x+c) = slope*b*(x-anchor_x).
        // Subtract exact view coordinates before rounding, so a large anchor
        // or translation cannot destroy a small displacement.
        arithmetic((|| {
            let distance = x.checked_sub(self.affine.anchor().x())?;
            (self.affine.anchor().y().bounded() + self.slope?.bounds.bounded() * distance).bounds()
        })())
    }

    fn integrate(
        &self,
        lo: NonNeg<N>,
        width: NonNeg<N>,
    ) -> MonotoneCalculatorResult<Bounds<NonNeg<N>>> {
        if width == NonNeg::zero() {
            return Ok(Bounds::exact(NonNeg::zero()));
        }
        let hi = arithmetic(lo.checked_add(width))?;
        self.require_input_domain(self.affine.anchor().x())?;
        self.require_input_domain(lo)?;
        self.require_input_domain(hi)?;
        arithmetic((|| {
            let anchor_area = self.affine.anchor().y().bounded() * width;
            if self.affine.y_scale() == NonNeg::zero() {
                return anchor_area.bounds();
            }
            let slope = self.slope?;
            let lo_distance = lo.checked_sub(self.affine.anchor().x())?;
            let hi_distance = hi.checked_sub(self.affine.anchor().x())?;
            let sum = lo_distance.checked_add(hi_distance)?;

            // span = (hi_distance^2 - lo_distance^2)/2, factored before
            // rounding. For a fixed hi it decreases with lo; for a fixed lo
            // it increases with hi. Every rounding step below preserves those
            // orders, as required by the default binary-search area solvers.
            let span = width.bounded() * sum / 2;
            let fraction_area = if slope.fraction.upper() == NonNeg::zero() {
                NonNeg::zero().bounded()
            } else {
                span * slope.fraction
            };
            // whole*width is exact. Fuse the next product with /2 so an
            // unscaled rectangle need not fit before its quotient does.
            let whole_area = slope.whole.bounded() * width * sum / 2;
            (anchor_area + whole_area + fraction_area).bounds()
        })())
    }
}

fn arithmetic<T>(result: MathResult<T>) -> MonotoneCalculatorResult<T> {
    result.map_err(|_| MonotoneCalculatorError::ArithmeticFailure)
}

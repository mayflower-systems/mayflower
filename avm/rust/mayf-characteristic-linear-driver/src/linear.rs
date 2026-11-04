use mayf_characteristic::{Characteristic, CharacteristicError};
use mayf_characteristic_linear::{LinearParams, LinearParamsError};
use mayf_fixed_number::{Bounded, Bounds, FixedNumber, MathResult, NonNeg, Pos};

/// The origin-anchored characteristic `f(x) = slope * x`, with positive slope.
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

impl<N: FixedNumber> Characteristic<N> for LinearCharacteristic<N> {
    fn y(&self, x: NonNeg<N>) -> Result<Bounds<NonNeg<N>>, CharacteristicError> {
        (self.slope().bounded() * x)
            .bounds()
            .map_err(|_| CharacteristicError::ArithmeticFailure)
    }

    fn integrate(
        &self,
        lo: NonNeg<N>,
        width: NonNeg<N>,
    ) -> Result<Bounds<NonNeg<N>>, CharacteristicError> {
        linear_area(self.slope(), lo, width).map_err(|_| CharacteristicError::ArithmeticFailure)
    }
}

fn linear_area<N: FixedNumber>(
    slope: Pos<N>,
    lo: NonNeg<N>,
    width: NonNeg<N>,
) -> MathResult<Bounds<NonNeg<N>>> {
    if width == NonNeg::zero() {
        return Ok(Bounds::exact(NonNeg::zero()));
    }
    let hi = lo.checked_add(width)?;
    let sum = lo.checked_add(hi);
    let times_midpoint = |weight: Bounded<N>| match sum {
        Ok(sum) => weight * sum / 2,
        // Near the end of the grid both coordinates can fit while their sum
        // overflows. Fuse each product with its division by two.
        Err(_) => weight * lo / 2 + weight * hi / 2,
    };

    // floor_grid(m*q) = floor(m)*q for grid spacing q. Splitting here lets
    // the whole slope multiply the exact width before any rounding. Only the
    // fractional slope multiplies an already rounded geometric area.
    let slope = slope.get();
    let whole = slope
        .checked_mul_floor(N::QUANTUM)?
        .checked_div_floor(N::QUANTUM)?;
    let fraction = slope.checked_sub(whole)?;
    let whole_area = times_midpoint(whole.bounded() * width);
    if fraction == N::ZERO {
        return whole_area.bounds();
    }
    (whole_area + times_midpoint(width.bounded()) * fraction).bounds()
}

#![no_std]

//! Persistent parameters for the origin-anchored linear characteristic.

use mayf_fixed_number::{FixedNumber, NonNeg};

/// A semantic failure while constructing linear parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinearParamsError {
    /// A negative slope would violate the increasing characteristic contract.
    NegativeSlope,
}

/// Parameters for the neutral characteristic `f(x) = slope * x`.
///
/// This is the persistable representation. Affine schedule state belongs to a
/// disposable calculator, not to these parameters. Construction of a concrete
/// characteristic asserts the semantic `Characteristic` contract; this
/// parameter type only rejects a slope that points downward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinearParams<N>
where
    N: FixedNumber,
{
    slope: NonNeg<N>,
}

impl<N> LinearParams<N>
where
    N: FixedNumber,
{
    pub const fn new(slope: NonNeg<N>) -> Self {
        Self { slope }
    }

    pub fn try_new(slope: N) -> Result<Self, LinearParamsError> {
        if slope < N::ZERO {
            return Err(LinearParamsError::NegativeSlope);
        }

        Ok(Self::new(
            NonNeg::new(slope).map_err(|_| LinearParamsError::NegativeSlope)?,
        ))
    }

    pub const fn slope(&self) -> NonNeg<N> {
        self.slope
    }
}

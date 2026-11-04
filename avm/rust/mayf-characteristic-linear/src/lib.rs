#![no_std]

//! Persistent parameters for the origin-anchored linear characteristic.

use mayf_fixed_number::{FixedNumber, Pos};

/// A semantic failure while constructing linear parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinearParamsError {
    /// A zero or negative slope would violate the increasing characteristic
    /// contract.
    NonPositiveSlope,
}

/// Parameters for the neutral characteristic `f(x) = slope * x`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinearParams<N>
where
    N: FixedNumber,
{
    slope: Pos<N>,
}

impl<N> LinearParams<N>
where
    N: FixedNumber,
{
    pub const fn new(slope: Pos<N>) -> Self {
        Self { slope }
    }

    pub fn try_new(slope: N) -> Result<Self, LinearParamsError> {
        Pos::new(slope)
            .map(Self::new)
            .map_err(|_| LinearParamsError::NonPositiveSlope)
    }

    pub const fn slope(&self) -> Pos<N> {
        self.slope
    }
}

#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use mayf_fixed_number::{Bounds, FixedNumber, NonNeg};

/// A neutral characteristic could not certify a value or integral.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacteristicError {
    /// The requested point or span is outside the function's supported domain.
    InputOutOfDomain,
    /// Checked arithmetic could not produce representable nonnegative bounds.
    ArithmeticFailure,
}

/// An immutable, smooth, increasing function on its supported nonnegative domain.
///
/// The function passes through the origin. Each successful result must enclose
/// the exact mathematical value on the chosen fixed-point grid.
pub trait Characteristic<N: FixedNumber> {
    /// Enclose `f(x)`. At the origin the result is exactly zero.
    fn y(&self, x: NonNeg<N>) -> Result<Bounds<NonNeg<N>>, CharacteristicError>;

    /// Enclose the integral of `f` over `[lo, lo + width]`.
    /// A zero width returns exact zero without evaluating the function.
    fn integrate(
        &self,
        lo: NonNeg<N>,
        width: NonNeg<N>,
    ) -> Result<Bounds<NonNeg<N>>, CharacteristicError>;
}

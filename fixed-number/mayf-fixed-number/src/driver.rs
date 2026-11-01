//! Restricted primitives for implementors of [`crate::FixedNumber`].
//!
//! Drivers are a mathematical trust boundary. A primitive must return the
//! greatest grid point below the exact result and report a discarded remainder
//! exactly. Errors are allowed outside the driver's documented supported range.

use crate::{DirectedRounding, FixedNumber, MathResult};

/// Capability passed by the library to driver primitives.
///
/// External drivers can implement methods accepting this token, but ordinary
/// consumers cannot construct it. It is not a security boundary against an
/// incorrectly implemented driver.
#[derive(Clone, Copy)]
pub struct Seal(pub(crate) ());

/// A native-grid floor and whether the exact result lies strictly above it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Floored<N> {
    pub value: N,
    pub inexact: bool,
}

impl<N: FixedNumber> Floored<N> {
    pub(crate) fn ceil(self) -> MathResult<N> {
        if self.inexact {
            self.value.checked_next_up()
        } else {
            Ok(self.value)
        }
    }

    pub(crate) fn rounded(self, rounding: DirectedRounding) -> MathResult<N> {
        match rounding {
            DirectedRounding::Floor => Ok(self.value),
            DirectedRounding::Ceil => self.ceil(),
        }
    }
}

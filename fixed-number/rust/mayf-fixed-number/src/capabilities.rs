//! Arithmetic capabilities, distinct from predicates on individual values.
//!
//! These traits describe semantic driver contracts; Rust does not prove their
//! mathematical laws. Implementations must be independently tested. They remain
//! open so drivers in other crates can participate.

use crate::{FixedNumber, MathResult};

/// A driver supporting negative values and signed directed arithmetic.
///
/// Addition, subtraction, negation, multiplication, division, and fused
/// multiply-divide must accept either sign whenever the rounded result is
/// representable. Floor and ceiling are mathematical directions, including
/// for negative results. Overflow and division by zero remain checked errors.
/// The bounded-expression primitives retain their nonnegative domain.
///
/// A generic function needing negative intermediates should require this trait,
/// while accepting `Pos<N>` or `NonNeg<N>` for constrained inputs. Unwrapping
/// those inputs returns the same `N`, with no representation conversion.
///
pub trait SignedFixed: FixedNumber {
    /// Exact additive inverse, failing when the result is outside the range.
    /// In particular, a bounded signed driver's minimum may not be negatable.
    fn checked_neg(self) -> MathResult<Self>;
}

/// A driver whose entire representable domain is nonnegative.
///
/// Subtraction below zero and conversion from negative values must fail rather
/// than wrap or saturate. This does not guarantee positivity: zero is valid.
/// A driver must not implement both this trait and `SignedFixed`.
/// Specialized formats without the full `FixedNumber` interface need not
/// implement either capability.
pub trait UnsignedFixed: FixedNumber {}

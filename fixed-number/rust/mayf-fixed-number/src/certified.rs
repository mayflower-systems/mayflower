//! Operations requiring mathematical certificates supplied by the caller.
//!
//! These are proof obligations, not Rust memory-safety `unsafe` contracts.
//! Shape checks cannot establish containment, ordering of the exact values, or
//! that two enclosures describe the same quantity. Document the justification
//! at each call site and test it independently of the endpoint implementation.

use crate::bounded::positive_difference;
use crate::{Bounded, Bounds, FixedNumber, MathError};

/// Refine an enclosure given a proof that its exact quantity is nonnegative.
/// Rounding slack below zero is removed. An entirely negative interval
/// contradicts the certificate and fails. Without that proof, use
/// `Bounds::try_nonnegative`, which rejects any negative endpoint.
pub fn nonnegative<N: FixedNumber>(bounds: Bounds<N>) -> Bounded<N> {
    if bounds.upper() < N::ZERO {
        Bounded::fail(MathError::Domain)
    } else {
        Bounded::from_result(Ok((bounds.lower().max(N::ZERO), bounds.upper())))
    }
}

/// Ordinary subtraction with a certificate that the exact `a >= b`.
///
/// This represents `a-b`; it does not replace the function with `max(a-b, 0)`.
/// Overlapping endpoints cannot prove the required ordering. Contradictory
/// endpoints fail; otherwise the certificate justifies clamping the lower edge.
pub fn sub_nonnegative<N: FixedNumber>(a: Bounded<N>, b: Bounded<N>) -> Bounded<N> {
    a.combine(b, |(al, au), (bl, bu)| {
        if au < bl {
            return Err(MathError::Domain);
        }
        Ok((positive_difference(al, bu)?, au.checked_sub(bl)?))
    })
}

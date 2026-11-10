//! Materialized, ordered intervals. Expression evaluation lives in `Bounded`.
//!
//! Signed arithmetic is limited to the operations used by affine translations
//! and exponential remainder formulas: sums, differences, exact products,
//! fused scalar scaling, negation, and error allowances.

use crate::{Bounded, FixedNumber, MathError, MathResult, NonNeg, Pos, SignedFixed};

/// Two ordered endpoints on a numeric grid.
///
/// Construction proves `lower <= upper`. The producer is responsible for proving
/// that these endpoints enclose the intended mathematical quantity. Arithmetic
/// preserves containment when its input enclosures are sound.
///
/// Endpoints are private, including for external numeric drivers:
///
/// ```compile_fail
/// use mayf_fixed_number::Bounds;
/// let invalid = Bounds { lower: 2, upper: 1 };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds<T> {
    lower: T,
    upper: T,
}

impl<T: Ord> Bounds<T> {
    pub fn new(lower: T, upper: T) -> MathResult<Self> {
        if lower > upper {
            Err(MathError::Domain)
        } else {
            Ok(Self { lower, upper })
        }
    }
}

impl<T: Copy + Ord> Bounds<T> {
    pub const fn exact(value: T) -> Self {
        Self {
            lower: value,
            upper: value,
        }
    }

    pub const fn lower(&self) -> T {
        self.lower
    }

    pub const fn upper(&self) -> T {
        self.upper
    }

    pub const fn into_edges(self) -> (T, T) {
        (self.lower, self.upper)
    }
}

impl<T: Copy + Ord> Bounds<T> {
    pub fn is_exact(&self) -> bool {
        self.lower == self.upper
    }

    pub fn contains(&self, value: T) -> bool {
        self.lower <= value && value <= self.upper
    }

    /// Intersect intervals. To retain a certified enclosure, both inputs must
    /// enclose the same mathematical quantity. Disjoint intervals fail.
    pub fn intersection(self, other: Self) -> MathResult<Self> {
        Self::new(self.lower.max(other.lower), self.upper.min(other.upper))
    }
}

impl<N: FixedNumber> Bounds<N> {
    pub fn checked_add(self, other: Self) -> MathResult<Self> {
        Self::new(
            self.lower.checked_add(other.lower)?,
            self.upper.checked_add(other.upper)?,
        )
    }

    pub fn checked_add_exact(self, value: N) -> MathResult<Self> {
        self.checked_add(Self::exact(value))
    }

    /// Refine both endpoints without clamping an uncertain or negative result.
    pub fn try_nonnegative(self) -> MathResult<Bounds<NonNeg<N>>> {
        Ok(Bounds {
            lower: NonNeg::new(self.lower)?,
            upper: NonNeg::new(self.upper)?,
        })
    }
}

impl<N: FixedNumber> Bounds<NonNeg<N>> {
    /// Forget the endpoint refinement, preserving their grid and order.
    pub fn into_raw(self) -> Bounds<N> {
        Bounds {
            lower: self.lower.get(),
            upper: self.upper.get(),
        }
    }

    /// Start a nonnegative expression from this materialized enclosure.
    pub fn bounded(self) -> Bounded<N> {
        self.into()
    }
}

impl<N: SignedFixed> Bounds<N> {
    pub fn checked_sub(self, other: Self) -> MathResult<Self> {
        Self::new(
            self.lower.checked_sub(other.upper)?,
            self.upper.checked_sub(other.lower)?,
        )
    }

    pub fn checked_sub_exact(self, value: N) -> MathResult<Self> {
        self.checked_sub(Self::exact(value))
    }

    pub fn checked_neg(self) -> MathResult<Self> {
        Self::new(self.upper.checked_neg()?, self.lower.checked_neg()?)
    }

    /// Enclose the product of two exact signed grid points.
    /// General signed interval multiplication is deliberately not provided.
    pub fn product(lhs: N, rhs: N) -> MathResult<Self> {
        Self::new(lhs.checked_mul_floor(rhs)?, lhs.checked_mul_ceil(rhs)?)
    }

    /// Scale an interval by an exact multiplier and positive divisor. Each
    /// endpoint uses one fused operation, including for negative multipliers.
    /// A representable quotient survives an overflowing standalone product.
    pub fn checked_mul_div(self, multiplier: N, divisor: Pos<N>) -> MathResult<Self> {
        let (lower, upper) = if multiplier >= N::ZERO {
            self.into_edges()
        } else {
            (self.upper, self.lower)
        };
        Self::new(
            lower.checked_mul_div_floor(multiplier, divisor.get())?,
            upper.checked_mul_div_ceil(multiplier, divisor.get())?,
        )
    }

    /// Add a symmetric absolute error allowance. Failure to represent either
    /// required endpoint is an error; the allowance is never silently clipped.
    pub fn checked_widen(self, allowance: NonNeg<N>) -> MathResult<Self> {
        Self::new(
            self.lower.checked_sub(allowance.get())?,
            self.upper.checked_add(allowance.get())?,
        )
    }
}

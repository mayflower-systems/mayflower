use core::ops::{Add, Div, Mul, Sub};

use crate::driver::Seal;
use crate::{AboveOne, Bounds, FixedNumber, MathError, MathResult, NonNeg, Pos};

type Edges<N> = (N, N);

#[derive(Clone, Copy, Debug)]
enum Expression<N> {
    Ready(Edges<N>),
    Product(Edges<N>, Edges<N>),
}

/// A nonnegative exact quantity enclosed by two outward-rounded grid points.
///
/// Arithmetic retains both edges; collapse only at a settlement or explicit
/// quantization boundary. Errors propagate through every operation, including
/// multiplication by zero.
///
/// ```
/// use mayf_fixed_number::{Bounded, Bounds, FixedNumber, MathResult, NonNeg};
/// fn anchored_area<N: FixedNumber>(
///     anchor_y: Bounds<NonNeg<N>>, width: NonNeg<N>, delta: Bounded<N>,
/// ) -> MathResult<Bounds<NonNeg<N>>> {
///     (anchor_y.bounded() * width + delta).bounds()
/// }
/// ```
///
/// Multiplication retains one pending product so `a * b / c` rounds the wide
/// quotient once. Assignment and helper calls preserve it. Other arithmetic
/// materializes that product; this is not an unlimited expression tree.
/// A successful evaluation guarantees containment, not the narrowest possible
/// enclosure or the nearest whole-unit rounding of the entire expression.
#[must_use = "bounded arithmetic may contain an error; evaluate or collapse the result"]
#[derive(Clone, Copy, Debug)]
pub struct Bounded<N> {
    state: MathResult<Expression<N>>,
}

impl<N: FixedNumber> Bounded<N> {
    /// Start an expression from an exact point. Negative points are rejected.
    pub fn exact(value: N) -> Self {
        Self::from_result(Ok((value, value)))
    }

    pub(crate) fn fail(error: MathError) -> Self {
        Self { state: Err(error) }
    }

    pub(crate) fn from_result(edges: MathResult<Edges<N>>) -> Self {
        let state = edges.and_then(|(lo, hi)| {
            if lo < N::ZERO || hi < lo {
                Err(MathError::Domain)
            } else {
                Ok(Expression::Ready((lo, hi)))
            }
        });
        Self { state }
    }

    pub(crate) fn combine(
        self,
        other: Self,
        operation: impl FnOnce(Edges<N>, Edges<N>) -> MathResult<Edges<N>>,
    ) -> Self {
        Self::from_result((|| operation(self.edges()?, other.edges()?))())
    }

    /// Evaluate any pending product without choosing an endpoint.
    ///
    /// This can change tightness and range of a subsequent division. In
    /// particular, an unfused product may overflow even when its quotient fits.
    pub fn materialize(self) -> Self {
        Self::from_result(self.edges())
    }

    /// Observe endpoints. Evaluates a copy without changing the original state.
    fn edges(self) -> MathResult<Edges<N>> {
        match self.state? {
            Expression::Ready(edges) => Ok(edges),
            Expression::Product(left, right) => quotient_edges(left, right, (N::ONE, N::ONE)),
        }
    }

    /// Materialize both endpoints as a validated nonnegative enclosure.
    /// This observes a copy and does not choose a rounding side or change a
    /// pending product in the original expression.
    pub fn bounds(self) -> MathResult<Bounds<NonNeg<N>>> {
        let (lower, upper) = self.edges()?;
        Bounds::new(lower, upper)?.try_nonnegative()
    }
}

pub(crate) fn positive_difference<N: FixedNumber>(a: N, b: N) -> MathResult<N> {
    if a <= b {
        Ok(N::ZERO)
    } else {
        a.checked_sub(b)
    }
}

fn quotient_edges<N: FixedNumber>(a: Edges<N>, b: Edges<N>, d: Edges<N>) -> MathResult<Edges<N>> {
    if d.0 == N::ZERO {
        return Err(if d.1 == N::ZERO {
            MathError::DivisionByZero
        } else {
            MathError::Domain
        });
    }
    let lower = a.0.mul_div_floor(b.0, d.1, Seal(()))?;
    let upper = if a.0 == a.1 && b.0 == b.1 && d.0 == d.1 {
        lower.ceil()?
    } else {
        a.1.mul_div_floor(b.1, d.0, Seal(()))?.ceil()?
    };
    if lower.value < N::ZERO || upper < lower.value {
        return Err(MathError::Domain);
    }
    Ok((lower.value, upper))
}

impl<N: FixedNumber, R: Into<Bounded<N>>> Add<R> for Bounded<N> {
    type Output = Self;
    fn add(self, rhs: R) -> Self {
        self.combine(rhs.into(), |(al, au), (bl, bu)| {
            Ok((al.checked_add(bl)?, au.checked_add(bu)?))
        })
    }
}

impl<N: FixedNumber, R: Into<Bounded<N>>> Sub<R> for Bounded<N> {
    type Output = Self;
    fn sub(self, rhs: R) -> Self {
        self.combine(rhs.into(), |(al, au), (bl, bu)| {
            if al < bu {
                return Err(MathError::Domain);
            }
            Ok((al.checked_sub(bu)?, au.checked_sub(bl)?))
        })
    }
}

impl<N: FixedNumber, R: Into<Bounded<N>>> Mul<R> for Bounded<N> {
    type Output = Self;
    fn mul(self, rhs: R) -> Self {
        let rhs = rhs.into();
        Self {
            state: (|| Ok(Expression::Product(self.edges()?, rhs.edges()?)))(),
        }
    }
}

impl<N: FixedNumber, R: Into<Bounded<N>>> Div<R> for Bounded<N> {
    type Output = Self;
    fn div(self, rhs: R) -> Self {
        let rhs = rhs.into();
        Self::from_result((|| {
            // Inspect an existing left error before evaluating the denominator.
            // Do not materialize a pending numerator product before fusion.
            let left = self.state?;
            let denominator = rhs.edges()?;
            match left {
                Expression::Ready(edges) => quotient_edges(edges, (N::ONE, N::ONE), denominator),
                Expression::Product(a, b) => quotient_edges(a, b, denominator),
            }
        })())
    }
}

impl<N: FixedNumber> From<N> for Bounded<N> {
    fn from(value: N) -> Self {
        Self::exact(value)
    }
}

impl<N: FixedNumber> From<Bounds<NonNeg<N>>> for Bounded<N> {
    fn from(bounds: Bounds<NonNeg<N>>) -> Self {
        Self::from_result(Ok(bounds.into_raw().into_edges()))
    }
}

macro_rules! refined_operands {
    ($($point:ident),* $(,)?) => { $(
        impl<N: FixedNumber> From<$point<N>> for Bounded<N> {
            fn from(value: $point<N>) -> Self { Self::exact(value.get()) }
        }
        impl<N: FixedNumber> Add<Bounded<N>> for $point<N> {
            type Output = Bounded<N>;
            fn add(self, rhs: Bounded<N>) -> Self::Output { Bounded::from(self) + rhs }
        }
        impl<N: FixedNumber> Sub<Bounded<N>> for $point<N> {
            type Output = Bounded<N>;
            fn sub(self, rhs: Bounded<N>) -> Self::Output { Bounded::from(self) - rhs }
        }
        impl<N: FixedNumber> Mul<Bounded<N>> for $point<N> {
            type Output = Bounded<N>;
            fn mul(self, rhs: Bounded<N>) -> Self::Output { Bounded::from(self) * rhs }
        }
        impl<N: FixedNumber> Div<Bounded<N>> for $point<N> {
            type Output = Bounded<N>;
            fn div(self, rhs: Bounded<N>) -> Self::Output { Bounded::from(self) / rhs }
        }
    )* };
}
refined_operands!(NonNeg, Pos, AboveOne);

macro_rules! integer_operands {
    ($($integer:ty),* $(,)?) => { $(
        impl<N: FixedNumber> From<$integer> for Bounded<N> {
            fn from(value: $integer) -> Self {
                // All supported integer operand types fit i128 exactly.
                let value = i128::from(value);
                if value < 0 { return Self::fail(MathError::Domain); }
                Self::from_result(N::checked_from_i128(value).map(|n| (n, n)))
            }
        }
        impl<N: FixedNumber> Add<Bounded<N>> for $integer {
            type Output = Bounded<N>;
            fn add(self, rhs: Bounded<N>) -> Self::Output { Bounded::from(self) + rhs }
        }
        impl<N: FixedNumber> Sub<Bounded<N>> for $integer {
            type Output = Bounded<N>;
            fn sub(self, rhs: Bounded<N>) -> Self::Output { Bounded::from(self) - rhs }
        }
        impl<N: FixedNumber> Mul<Bounded<N>> for $integer {
            type Output = Bounded<N>;
            fn mul(self, rhs: Bounded<N>) -> Self::Output { Bounded::from(self) * rhs }
        }
        impl<N: FixedNumber> Div<Bounded<N>> for $integer {
            type Output = Bounded<N>;
            fn div(self, rhs: Bounded<N>) -> Self::Output { Bounded::from(self) / rhs }
        }
    )* };
}
integer_operands!(i32, u32, u64);

//! Branded numbers: values carrying compile-time proof of a numeric predicate.

use crate::{Bounded, DirectedRounding, FixedNumber, MathError, MathResult, UnsignedFixed};

/// A number proven `>= 0` at construction.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NonNeg<N>(N);

/// A number proven `> 0` at construction.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos<N>(N);

/// A number proven `> 1` at construction.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AboveOne<N>(N);

impl<N: UnsignedFixed> NonNeg<N> {
    /// The unsigned driver proves nonnegativity without a runtime sign check.
    pub const fn from_unsigned(value: N) -> Self {
        Self(value)
    }
}

macro_rules! bounded_refinements {
    ($($name:ident),* $(,)?) => { $(
        impl<N: FixedNumber> $name<N> {
            /// Lift this refined grid point into a nonnegative expression.
            pub fn bounded(self) -> Bounded<N> { self.into() }
        }
    )* };
}
bounded_refinements!(NonNeg, Pos, AboveOne);

impl<N: FixedNumber> NonNeg<N> {
    /// Validate `value >= 0`.
    pub fn new(value: N) -> MathResult<Self> {
        if value >= N::ZERO {
            Ok(Self(value))
        } else {
            Err(MathError::Domain)
        }
    }

    /// Convert an unsigned whole amount, checking the driver's range.
    pub fn from_u64(value: u64) -> MathResult<Self> {
        N::checked_from_u64(value).map(Self)
    }

    /// The additive identity `0`, which is non-negative.
    pub fn zero() -> Self {
        Self(N::ZERO)
    }

    /// The multiplicative identity, known nonnegative by the driver contract.
    pub fn one() -> Self {
        Self(N::ONE)
    }

    /// Drop to the raw driver `N`.
    pub fn get(self) -> N {
        self.0
    }
}

impl<N: FixedNumber> Pos<N> {
    /// Validate `value > 0`.
    pub fn new(value: N) -> MathResult<Self> {
        if value > N::ZERO {
            Ok(Self(value))
        } else {
            Err(MathError::Domain)
        }
    }

    /// The multiplicative identity `1`, which is positive.
    pub fn one() -> Self {
        Self(N::ONE)
    }

    pub fn from_i32(value: i32) -> MathResult<Self> {
        Self::new(N::from_i32(value)?)
    }

    /// Drop to the raw driver `N`.
    pub fn get(self) -> N {
        self.0
    }
}

impl<N: FixedNumber> AboveOne<N> {
    /// Validate `value > 1`.
    pub fn new(value: N) -> MathResult<Self> {
        if value > N::ONE {
            Ok(Self(value))
        } else {
            Err(MathError::Domain)
        }
    }

    /// Drop to the raw driver `N`.
    pub fn get(self) -> N {
        self.0
    }
}

impl<N: FixedNumber> From<Pos<N>> for NonNeg<N> {
    fn from(x: Pos<N>) -> Self {
        Self(x.0)
    }
}

impl<N: FixedNumber> From<AboveOne<N>> for NonNeg<N> {
    fn from(x: AboveOne<N>) -> Self {
        Self(x.0)
    }
}

impl<N: FixedNumber> From<AboveOne<N>> for Pos<N> {
    fn from(x: AboveOne<N>) -> Self {
        Self(x.0)
    }
}

impl<N: FixedNumber> TryFrom<NonNeg<N>> for Pos<N> {
    type Error = MathError;
    fn try_from(x: NonNeg<N>) -> MathResult<Self> {
        Self::new(x.0)
    }
}

impl<N: FixedNumber> TryFrom<NonNeg<N>> for AboveOne<N> {
    type Error = MathError;
    fn try_from(x: NonNeg<N>) -> MathResult<Self> {
        Self::new(x.0)
    }
}

impl<N: FixedNumber> TryFrom<Pos<N>> for AboveOne<N> {
    type Error = MathError;
    fn try_from(x: Pos<N>) -> MathResult<Self> {
        Self::new(x.0)
    }
}

impl<N: FixedNumber> NonNeg<N> {
    /// Sum of two non-negatives is non-negative.
    pub fn checked_add(self, other: impl Into<Self>) -> MathResult<Self> {
        self.0.checked_add(other.into().0).map(Self)
    }

    /// The difference is non-negative only when `self >= other`.
    pub fn checked_sub(self, other: impl Into<Self>) -> MathResult<Self> {
        let other = other.into();
        if self.0 < other.0 {
            return Err(MathError::Domain);
        }

        let result = self.0.checked_sub(other.0)?;
        Ok(Self(result))
    }

    pub fn checked_mul_round(
        self,
        other: impl Into<Self>,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        self.0.checked_mul_round(other.into().0, rounding).map(Self)
    }

    pub fn checked_mul_floor(self, other: impl Into<Self>) -> MathResult<Self> {
        self.checked_mul_round(other, DirectedRounding::Floor)
    }

    pub fn checked_mul_ceil(self, other: impl Into<Self>) -> MathResult<Self> {
        self.checked_mul_round(other, DirectedRounding::Ceil)
    }

    /// A non-negative divided by a positive stays non-negative and cannot
    /// divide by zero.
    pub fn checked_div_round(
        self,
        divisor: Pos<N>,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        self.0.checked_div_round(divisor.0, rounding).map(Self)
    }

    pub fn checked_div_floor(self, divisor: Pos<N>) -> MathResult<Self> {
        self.checked_div_round(divisor, DirectedRounding::Floor)
    }

    pub fn checked_div_ceil(self, divisor: Pos<N>) -> MathResult<Self> {
        self.checked_div_round(divisor, DirectedRounding::Ceil)
    }

    pub fn checked_mul_div_round(
        self,
        multiplier: impl Into<Self>,
        divisor: Pos<N>,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        self.0
            .checked_mul_div_round(multiplier.into().0, divisor.0, rounding)
            .map(Self)
    }

    pub fn checked_mul_div_floor(
        self,
        multiplier: impl Into<Self>,
        divisor: Pos<N>,
    ) -> MathResult<Self> {
        self.checked_mul_div_round(multiplier, divisor, DirectedRounding::Floor)
    }

    pub fn checked_mul_div_ceil(
        self,
        multiplier: impl Into<Self>,
        divisor: Pos<N>,
    ) -> MathResult<Self> {
        self.checked_mul_div_round(multiplier, divisor, DirectedRounding::Ceil)
    }
}

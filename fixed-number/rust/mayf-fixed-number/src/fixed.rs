//! Fixed-point arithmetic driver traits and shared numeric result types.

use core::num::NonZeroU128;

use crate::Bounds;
use crate::driver::{Floored, Seal};

/// Arithmetic failures from the fixed-point driver / refinement layer.
///
/// Every variant is a checked failure surfaced by [`FixedNumber`] or an
/// extension trait. Higher-level modules should map these into their own
/// semantic errors instead of exposing this enum directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathError {
    Overflow,
    DivisionByZero,
    Inexact,
    NegativeToUnsigned,
    Domain,
    UnsupportedRange,
}

pub type MathResult<T> = Result<T, MathError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectedRounding {
    /// Greatest representable value no greater than the exact result.
    Floor,
    /// Smallest representable value no less than the exact result.
    Ceil,
}

/// Minimal fixed-point interface market math is generic over.
///
/// Every operation is checked: overflow, underflow, division-by-zero, and
/// unsupported conversions return [`MathError`] rather than panicking or
/// wrapping.
///
/// `QUANTUM` is the smallest positive spacing between adjacent representable
/// values for this type. Every value of a given driver type must live on that
/// single grid.
///
/// `Eq` is required, not just `PartialEq`: a driver must have reflexive
/// equality. This is the type-level statement that raw floating-point values
/// have no place in this abstraction.
///
/// Drivers implement the exact point operations and two restricted
/// floor/remainder primitives. Directed methods have nonnegative defaults using
/// those primitives. A [`crate::SignedFixed`] implementation overrides the
/// directed methods to support either sign and separately provides negation.
/// Bounded expressions always use the nonnegative primitives.
///
/// Algorithms that create negative intermediates must require `SignedFixed`;
/// `FixedNumber` alone does not promise that negative values are representable.
pub trait FixedNumber: Copy + Eq + Ord + Sized {
    /// Additive identity `0`.
    const ZERO: Self;

    /// Multiplicative identity `1`.
    const ONE: Self;

    /// Smallest positive distance between adjacent representable values.
    const QUANTUM: Self;

    /// Exact conversion for integer constants used by formulas.
    fn checked_from_i128(value: i128) -> MathResult<Self>;

    /// Widen an unsigned whole-unit amount exactly onto this type's grid.
    ///
    /// Returns an error when the driver's range cannot represent the amount.
    fn checked_from_u64(value: u64) -> MathResult<Self> {
        Self::checked_from_i128(i128::from(value))
    }

    fn from_i32(value: i32) -> MathResult<Self> {
        Self::checked_from_i128(i128::from(value))
    }

    fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    fn checked_add(self, rhs: Self) -> MathResult<Self>;
    fn checked_sub(self, rhs: Self) -> MathResult<Self>;

    /// Lift an exact nonnegative point into an outward-rounded expression.
    /// A negative input becomes a sticky domain error.
    fn bounded(self) -> crate::Bounded<Self> {
        crate::Bounded::exact(self)
    }

    /// Floor the exact `self * mul / div` once, on the native grid.
    ///
    /// Operands must be nonnegative and `div > 0`. Reject invalid domains.
    /// Use a wide intermediate: an overflowing unfused product must not reject
    /// a representable final quotient. `inexact` is true iff a nonzero remainder
    /// was discarded. This primitive and `ratio_floor` power bounded chains;
    /// optimized drivers need implement no separate ceiling primitive.
    fn mul_div_floor(self, mul: Self, div: Self, seal: Seal) -> MathResult<Floored<Self>>;

    /// Floor an unsigned rational onto the native grid, with an exact remainder
    /// flag. Numerator and denominator have the full `u128` range; a driver may
    /// return `UnsupportedRange` for explicitly documented narrower support.
    fn ratio_floor(
        numerator: u128,
        denominator: NonZeroU128,
        seal: Seal,
    ) -> MathResult<Floored<Self>>;

    /// Round the exact product to this type's quantum grid.
    fn checked_mul_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        self.mul_div_floor(rhs, Self::ONE, Seal(()))?
            .rounded(rounding)
    }

    /// Round the exact quotient to this type's quantum grid.
    fn checked_div_round(self, rhs: Self, rounding: DirectedRounding) -> MathResult<Self> {
        self.mul_div_floor(Self::ONE, rhs, Seal(()))?
            .rounded(rounding)
    }

    /// Compute `(self * multiplier) / divisor` with one final rounding step.
    fn checked_mul_div_round(
        self,
        multiplier: Self,
        divisor: Self,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        self.mul_div_floor(multiplier, divisor, Seal(()))?
            .rounded(rounding)
    }

    /// Construct `numerator / denominator` directly on this type's grid.
    fn checked_from_ratio_round(
        numerator: i128,
        denominator: NonZeroU128,
        rounding: DirectedRounding,
    ) -> MathResult<Self> {
        let numerator = u128::try_from(numerator).map_err(|_| MathError::Domain)?;
        Self::ratio_floor(numerator, denominator, Seal(()))?.rounded(rounding)
    }

    fn checked_mul_floor(self, rhs: Self) -> MathResult<Self> {
        self.checked_mul_round(rhs, DirectedRounding::Floor)
    }

    fn checked_mul_ceil(self, rhs: Self) -> MathResult<Self> {
        self.checked_mul_round(rhs, DirectedRounding::Ceil)
    }

    fn checked_div_floor(self, rhs: Self) -> MathResult<Self> {
        self.checked_div_round(rhs, DirectedRounding::Floor)
    }

    fn checked_div_ceil(self, rhs: Self) -> MathResult<Self> {
        self.checked_div_round(rhs, DirectedRounding::Ceil)
    }

    fn checked_mul_div_floor(self, multiplier: Self, divisor: Self) -> MathResult<Self> {
        self.checked_mul_div_round(multiplier, divisor, DirectedRounding::Floor)
    }

    fn checked_mul_div_ceil(self, multiplier: Self, divisor: Self) -> MathResult<Self> {
        self.checked_mul_div_round(multiplier, divisor, DirectedRounding::Ceil)
    }

    /// Bound a nonnegative integer power from above by exponentiation by squaring.
    ///
    /// Each multiplication rounds upward. The result is an upper bound on the
    /// exact power, but need not be its nearest ceiling: intermediate products
    /// also round. Multiplications consume exponent bits from least to most
    /// significant, multiplying the accumulator before squaring the base.
    ///
    /// Every value to the power zero, including zero, returns `ONE`. Negative
    /// bases with a positive exponent return [`MathError::Domain`], because
    /// multiplying negative bounds can reverse their rounding direction.
    fn checked_pow_ceil(self, mut exponent: u32) -> MathResult<Self> {
        if exponent == 0 {
            return Ok(Self::ONE);
        }
        if self < Self::ZERO {
            return Err(MathError::Domain);
        }

        let mut base = self;
        let mut result = Self::ONE;
        while exponent > 0 {
            if exponent & 1 == 1 {
                result = result.checked_mul_ceil(base)?;
            }
            exponent >>= 1;
            if exponent > 0 {
                base = base.checked_mul_ceil(base)?;
            }
        }
        Ok(result)
    }

    fn checked_next_up(self) -> MathResult<Self> {
        self.checked_add(Self::QUANTUM)
    }

    fn checked_next_down(self) -> MathResult<Self> {
        self.checked_sub(Self::QUANTUM)
    }
}

/// A fixed-point driver that can floor a value to a whole `u64` amount.
///
/// This is deliberately separate from [`FixedNumber`]. Most protocol math is
/// expressed entirely on the driver's native quantum and does not need to
/// know where whole units lie. Token settlement is different: SPL custody can
/// move only integral base units, so a payout must be rounded before it is
/// removed from an epoch's accounting.
pub trait IntegralRounding: FixedNumber {
    /// Round down to whole units and extract an unsigned amount.
    ///
    /// Every negative input, including a negative fraction, returns
    /// [`MathError::NegativeToUnsigned`]. A floor above `u64::MAX` returns
    /// [`MathError::Overflow`]; `u64::MAX` plus a fraction still floors to
    /// `u64::MAX`. No rounded intermediate need fit the driver's own grid.
    fn checked_floor_to_u64(self) -> MathResult<u64>;
}

/// Fixed-point numeric engines that can certify bounds for `exp(x) - 1`.
///
/// This is the only transcendental primitive required by the reference
/// implementation. Other implementations can use additional private primitives
/// or hand-rolled approximations without changing this public contract.
pub trait BoundedExponential: FixedNumber {
    fn checked_exp_m1_bounds(self) -> MathResult<Bounds<Self>>;
}

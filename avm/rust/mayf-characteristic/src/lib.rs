#![no_std]

//! Characteristic definitions and their disposable anchored affine calculators.

use mayf_fixed_number::{Bounds, FixedNumber, NonNeg, Pos, SignedFixed};
use mayf_monotone::MonotoneCalculator;

/// A semantic failure while validating an affine calculation view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AffineTransformError {
    NonPositiveHorizontalScale,
    NegativeVerticalScale,
    NegativeAnchorInput,
    NegativeAnchorOutput,
}

pub type AffineTransformResult<T> = Result<T, AffineTransformError>;

/// A certified point through which an affine calculation view passes.
///
/// The output is bounded because a schedule may derive this anchor from a
/// previous certified calculation, such as the ramp price at its endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AffineAnchor<N>
where
    N: FixedNumber,
{
    x: NonNeg<N>,
    y: Bounds<NonNeg<N>>,
}

impl<N> AffineAnchor<N>
where
    N: FixedNumber,
{
    /// Refined inputs already prove the anchor's numeric invariants.
    pub const fn new(x: NonNeg<N>, y: Bounds<NonNeg<N>>) -> Self {
        Self { x, y }
    }

    pub fn exact(x: NonNeg<N>, y: NonNeg<N>) -> Self {
        Self {
            x,
            y: Bounds::exact(y),
        }
    }

    pub fn try_new(x: N, y: Bounds<N>) -> AffineTransformResult<Self> {
        let x = NonNeg::new(x).map_err(|_| AffineTransformError::NegativeAnchorInput)?;
        let y = y
            .try_nonnegative()
            .map_err(|_| AffineTransformError::NegativeAnchorOutput)?;
        Ok(Self::new(x, y))
    }

    pub const fn x(&self) -> NonNeg<N> {
        self.x
    }

    pub const fn y(&self) -> Bounds<NonNeg<N>> {
        self.y
    }
}

/// An anchored affine view of a neutral characteristic `f`.
///
/// The calculator evaluates:
///
/// ```text
/// u(x) = x_scale*x + x_translation
/// g(x) = anchor_y + y_scale * (f(u(x)) - f(u(anchor_x)))
/// ```
///
/// Expanding the second line gives the classic `a*f(b*x+c)+d` form. Keeping
/// the anchor instead of the derived `d` avoids cancellation at schedule
/// boundaries and makes continuity a construction invariant. The supported
/// domain is `x >= anchor_x`, where increasingness proves the difference is
/// non-negative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AffineTransform<N>
where
    N: SignedFixed,
{
    x_scale: Pos<N>,
    x_translation: N,
    y_scale: NonNeg<N>,
    anchor: AffineAnchor<N>,
}

impl<N> AffineTransform<N>
where
    N: SignedFixed,
{
    pub const fn new(
        x_scale: Pos<N>,
        x_translation: N,
        y_scale: NonNeg<N>,
        anchor: AffineAnchor<N>,
    ) -> Self {
        Self {
            x_scale,
            x_translation,
            y_scale,
            anchor,
        }
    }

    pub fn try_new(
        x_scale: N,
        x_translation: N,
        y_scale: N,
        anchor_x: N,
        anchor_y: Bounds<N>,
    ) -> AffineTransformResult<Self> {
        if x_scale <= N::ZERO {
            return Err(AffineTransformError::NonPositiveHorizontalScale);
        }
        if y_scale < N::ZERO {
            return Err(AffineTransformError::NegativeVerticalScale);
        }

        Ok(Self::new(
            Pos::new(x_scale).map_err(|_| AffineTransformError::NonPositiveHorizontalScale)?,
            x_translation,
            NonNeg::new(y_scale).map_err(|_| AffineTransformError::NegativeVerticalScale)?,
            AffineAnchor::try_new(anchor_x, anchor_y)?,
        ))
    }

    /// The neutral view `g(x) = f(x)` over the non-negative domain.
    pub fn identity() -> Self {
        Self::new(
            Pos::one(),
            N::ZERO,
            NonNeg::one(),
            AffineAnchor::exact(NonNeg::zero(), NonNeg::zero()),
        )
    }

    pub fn is_identity(&self) -> bool {
        self.x_scale.get() == N::ONE
            && self.x_translation == N::ZERO
            && self.y_scale.get() == N::ONE
            && self.anchor.x == NonNeg::zero()
            && self.anchor.y.lower() == NonNeg::zero()
            && self.anchor.y.upper() == NonNeg::zero()
    }

    pub const fn x_scale(&self) -> Pos<N> {
        self.x_scale
    }

    pub const fn x_translation(&self) -> N {
        self.x_translation
    }

    pub const fn y_scale(&self) -> NonNeg<N> {
        self.y_scale
    }

    pub const fn anchor(&self) -> AffineAnchor<N> {
        self.anchor
    }
}

/// A neutral characteristic shape that compiles a disposable affine view
/// without mutating its persistent parameters.
///
/// Implementing this trait asserts that the neutral function passes through
/// `(0, 0)` and is increasing, smooth, and differentiable over its supported
/// non-negative domain. These are semantic obligations of each implementation;
/// callers rely on them rather than attempting to establish them by sampling.
pub trait Characteristic<N>
where
    N: SignedFixed,
{
    type Calculator: MonotoneCalculator<N>;

    fn calculator(&self, affine: AffineTransform<N>) -> Self::Calculator;
}

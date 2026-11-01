#![no_std]

//! Calculation contract for non-negative, monotone non-decreasing functions.
//!
//! This crate owns semantic calculation failures. Implementations may use a
//! lower-level numeric driver internally, but must map its failures before they
//! cross this interface.

use mayf_fixed_number::{Bounds, FixedNumber, NonNeg};

/// A semantic failure while evaluating or solving a monotone function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonotoneCalculatorError {
    /// An input or transformed input lies outside the function's domain.
    InputOutOfDomain,

    /// An interval is reversed or inconsistent with a supplied solve limit.
    InvalidInterval,

    /// The calculated output enclosure is not wholly non-negative.
    ///
    /// Implementations must return this error instead of clamping a negative
    /// lower edge to zero. That keeps an invalid transform or uncertain sign
    /// from entering downstream price and reserve calculations.
    NegativeOutput,

    /// Checked arithmetic could not produce a representable result.
    ArithmeticFailure,
}

pub type MonotoneCalculatorResult<T> = Result<T, MonotoneCalculatorError>;

/// Prove that a numeric output enclosure is entirely non-negative.
///
/// An enclosure that straddles zero is rejected rather than clamped. Although
/// its exact value might be non-negative, the calculator has not proved that
/// fact tightly enough for the value to enter downstream calculations.
pub fn require_nonnegative<N>(bounds: Bounds<N>) -> MonotoneCalculatorResult<Bounds<NonNeg<N>>>
where
    N: FixedNumber,
{
    bounds
        .try_nonnegative()
        .map_err(|_| MonotoneCalculatorError::NegativeOutput)
}

/// The result of solving for an endpoint from a requested integral area.
///
/// `area` is the achieved safe-side area, not necessarily the requested area:
///
/// - An upward solve returns an upper-bounded area no greater than the request.
/// - A downward solve returns a lower-bounded area no less than the request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AreaSolution<N>
where
    N: FixedNumber,
{
    /// The solved supply coordinate.
    pub endpoint: NonNeg<N>,

    /// The certified area enclosed between the fixed anchor and `endpoint`.
    pub area: NonNeg<N>,
}

/// A disposable evaluator for one fully configured monotone function.
///
/// The calculator may carry derived constants or a transient affine transform,
/// but it does not own persistent market state. Every returned enclosure must
/// contain the exact result and be entirely non-negative.
pub trait MonotoneCalculator<N>
where
    N: FixedNumber,
{
    /// Enclose the function value at `x`.
    fn y(&self, x: NonNeg<N>) -> MonotoneCalculatorResult<Bounds<NonNeg<N>>>;

    /// Enclose the integral over `[lo, lo + width]`.
    ///
    /// A zero width returns the exact zero enclosure.
    fn integrate(
        &self,
        lo: NonNeg<N>,
        width: NonNeg<N>,
    ) -> MonotoneCalculatorResult<Bounds<NonNeg<N>>>;

    /// Find the greatest endpoint whose upper-bounded area is at most `area`.
    ///
    /// `hi_limit` confines the search to a finite segment when supplied. A zero
    /// area returns `lo`. Otherwise, returns `None` when no positive move from
    /// `lo` is possible.
    fn max_hi_for_area_at_or_below(
        &self,
        lo: NonNeg<N>,
        hi_limit: Option<NonNeg<N>>,
        area: NonNeg<N>,
    ) -> MonotoneCalculatorResult<Option<AreaSolution<N>>> {
        solve_max_hi(self, lo, hi_limit, area)
    }

    /// Find the greatest endpoint whose lower-bounded area is at least `area`.
    ///
    /// The search is confined to `[lo_limit, hi]`. A zero area returns `hi`.
    /// Otherwise, returns `None` when the request is unreachable in that
    /// interval.
    fn max_lo_for_area_at_or_above(
        &self,
        lo_limit: NonNeg<N>,
        hi: NonNeg<N>,
        area: NonNeg<N>,
    ) -> MonotoneCalculatorResult<Option<AreaSolution<N>>> {
        solve_max_lo(self, lo_limit, hi, area)
    }
}

fn solve_max_hi<N, C>(
    calculator: &C,
    lo: NonNeg<N>,
    hi_limit: Option<NonNeg<N>>,
    area: NonNeg<N>,
) -> MonotoneCalculatorResult<Option<AreaSolution<N>>>
where
    N: FixedNumber,
    C: MonotoneCalculator<N> + ?Sized,
{
    if hi_limit.is_some_and(|limit| limit < lo) {
        return Err(MonotoneCalculatorError::InvalidInterval);
    }
    if area == NonNeg::zero() {
        return Ok(Some(zero_area_solution(lo)));
    }
    let mut can_afford = |hi| {
        let width = positive_difference(lo, hi)?;
        Ok(integrate_bounds(calculator, lo, width)?.upper().get() <= area.get())
    };
    let too_expensive = match hi_limit {
        Some(limit) => {
            if limit == lo {
                return Ok(None);
            }
            if can_afford(limit)? {
                return Ok(Some(AreaSolution {
                    endpoint: limit,
                    area: integrate_bounds(calculator, lo, positive_difference(lo, limit)?)?
                        .upper(),
                }));
            }
            limit
        }
        None => first_violation_above(lo, &mut can_afford)?,
    };
    let endpoint = greatest_satisfying(lo, too_expensive, &mut can_afford)?;
    if endpoint <= lo {
        return Ok(None);
    }

    Ok(Some(AreaSolution {
        endpoint,
        area: integrate_bounds(calculator, lo, positive_difference(lo, endpoint)?)?.upper(),
    }))
}

fn solve_max_lo<N, C>(
    calculator: &C,
    lo_limit: NonNeg<N>,
    hi: NonNeg<N>,
    area: NonNeg<N>,
) -> MonotoneCalculatorResult<Option<AreaSolution<N>>>
where
    N: FixedNumber,
    C: MonotoneCalculator<N> + ?Sized,
{
    if lo_limit > hi {
        return Err(MonotoneCalculatorError::InvalidInterval);
    }
    if area == NonNeg::zero() {
        return Ok(Some(zero_area_solution(hi)));
    }
    if lo_limit == hi {
        return Ok(None);
    }

    let mut meets_target = |lo| {
        let width = positive_difference(lo, hi)?;
        Ok(integrate_bounds(calculator, lo, width)?.lower().get() >= area.get())
    };
    if !meets_target(lo_limit)? {
        return Ok(None);
    }

    let endpoint = greatest_satisfying(lo_limit, hi, &mut meets_target)?;
    if endpoint >= hi {
        return Ok(None);
    }

    Ok(Some(AreaSolution {
        endpoint,
        area: integrate_bounds(calculator, endpoint, positive_difference(endpoint, hi)?)?.lower(),
    }))
}

fn integrate_bounds<N, C>(
    calculator: &C,
    lo: NonNeg<N>,
    width: NonNeg<N>,
) -> MonotoneCalculatorResult<Bounds<NonNeg<N>>>
where
    N: FixedNumber,
    C: MonotoneCalculator<N> + ?Sized,
{
    if width == NonNeg::zero() {
        return Ok(Bounds::exact(NonNeg::zero()));
    }
    calculator.integrate(lo, width)
}

fn positive_difference<N>(lo: NonNeg<N>, hi: NonNeg<N>) -> MonotoneCalculatorResult<NonNeg<N>>
where
    N: FixedNumber,
{
    if hi <= lo {
        return Err(MonotoneCalculatorError::InvalidInterval);
    }

    let width = hi
        .get()
        .checked_sub(lo.get())
        .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
    NonNeg::new(width).map_err(|_| MonotoneCalculatorError::ArithmeticFailure)
}

fn zero_area_solution<N>(endpoint: NonNeg<N>) -> AreaSolution<N>
where
    N: FixedNumber,
{
    AreaSolution {
        endpoint,
        area: NonNeg::zero(),
    }
}

fn greatest_satisfying<N, F>(
    mut satisfying: NonNeg<N>,
    mut violating: NonNeg<N>,
    predicate: &mut F,
) -> MonotoneCalculatorResult<NonNeg<N>>
where
    N: FixedNumber,
    F: FnMut(NonNeg<N>) -> MonotoneCalculatorResult<bool>,
{
    if satisfying >= violating {
        return Err(MonotoneCalculatorError::InvalidInterval);
    }

    let two = N::from_i32(2).map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
    while violating
        .get()
        .checked_sub(satisfying.get())
        .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?
        > N::QUANTUM
    {
        let distance = violating
            .get()
            .checked_sub(satisfying.get())
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let offset = distance
            .checked_div_floor(two)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let middle = NonNeg::new(
            satisfying
                .get()
                .checked_add(offset)
                .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?,
        )
        .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        if predicate(middle)? {
            satisfying = middle;
        } else {
            violating = middle;
        }
    }

    Ok(satisfying)
}

fn first_violation_above<N, F>(
    anchor: NonNeg<N>,
    predicate: &mut F,
) -> MonotoneCalculatorResult<NonNeg<N>>
where
    N: FixedNumber,
    F: FnMut(NonNeg<N>) -> MonotoneCalculatorResult<bool>,
{
    let mut distance = N::QUANTUM;
    loop {
        let candidate = NonNeg::new(
            anchor
                .get()
                .checked_add(distance)
                .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?,
        )
        .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        if !predicate(candidate)? {
            return Ok(candidate);
        }
        distance = distance
            .checked_add(distance)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
    }
}

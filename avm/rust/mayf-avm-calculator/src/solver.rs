//! Optional iterative proposal builders for clients and simulations.
//!
//! These searches make repeated characteristic calls. Program actions should
//! receive their proposals and use `AvmCalculator::verify_floor_raise` instead.

use core::num::NonZeroU32;

use mayf_fixed_number::{Bounds, NonNeg, Pos, SignedFixed};

use crate::calculator::{evaluate_spans, main_price, point_request, ramp_price};
use crate::market::FloorRaiseEvaluation;
use crate::{
    AvmCalculator, CharacteristicEngine, Error, FloorRaisePolicy, FloorRaiseProposal,
    FloorRaiseQuote, MarketState,
};

/// Accuracy policy and total characteristic-batch budget for one search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloorRaiseSearch<N: SignedFixed> {
    /// Includes nested searches and the final independent verification.
    pub max_iterations: NonZeroU32,
    pub max_floor_join_gap: NonNeg<N>,
    pub max_area_decrease: NonNeg<N>,
}

/// A proposal accepted by the ordinary one-sided floor-raise verifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloorRaiseSolution<N: SignedFixed> {
    pub proposal: FloorRaiseProposal<N>,
    pub quote: FloorRaiseQuote<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolverError {
    Calculator(Error),
    /// The supplied end precedes the old end or exceeds current progress.
    InvalidRange,
    /// The requested increase cannot be bracketed within the supplied range.
    NoSolution,
    /// The native grid or engine bounds cannot certify the requested accuracy.
    PrecisionLimit,
    IterationLimit,
    ArithmeticFailure,
}

type Result<T> = core::result::Result<T, SolverError>;

/// Find a ramp for a specified positive floor increment, preserving area within
/// `search.max_area_decrease`. The end never exceeds `max_ramp_end`.
///
/// The outer bisection moves ramp end; an inner bisection finds the floor
/// intersection. No derivative or characteristic-specific inverse is needed.
pub fn solve_floor_increase<N: SignedFixed>(
    calculator: &AvmCalculator<N>,
    state: &MarketState<N>,
    floor_increase_by: Pos<N>,
    max_ramp_end: NonNeg<N>,
    search: FloorRaiseSearch<N>,
    engine: &dyn CharacteristicEngine<N>,
) -> Result<FloorRaiseSolution<N>> {
    let mut context = Search::new(calculator, state, max_ramp_end, search, engine)?;
    let floor = state
        .segmentation()
        .floor()
        .checked_add(floor_increase_by)
        .map_err(|_| SolverError::ArithmeticFailure)?;
    // An empty physical floor tail can absorb an existing upward join gap
    // without moving either boundary. Check this endpoint before searching.
    if state.floor_tail().map_err(SolverError::Calculator)? == NonNeg::zero() {
        let unchanged = FloorRaiseProposal {
            floor_increase_by,
            ramp_end: state.segmentation().ramp_end(),
            ramp_width: state.segmentation().ramp_width(),
        };
        let evaluation = context.evaluate(unchanged)?;
        if context.accepts(&evaluation)? {
            return context.finish(unchanged);
        }
    }
    let mut lo = state.segmentation().ramp_end();
    let mut hi = max_ramp_end;
    let mut end = hi;
    loop {
        let ramp = context.ramp(end)?;
        match context.intersection(&ramp, floor)? {
            None => {
                if end == max_ramp_end {
                    return Err(SolverError::NoSolution);
                }
                lo = end;
            }
            Some(intersection) => {
                let proposal = proposal(floor_increase_by, end, intersection.upper())?;
                let evaluation = context.evaluate(proposal)?;
                if context.accepts(&evaluation)? {
                    return context.finish(proposal);
                }
                // Moving the stored start past the true intersection removes a
                // nonnegative sliver. Enclose it before choosing the direction
                // of the ideal, continuous area residual.
                let width = intersection
                    .upper()
                    .checked_sub(intersection.lower())
                    .map_err(|_| SolverError::ArithmeticFailure)?;
                let sliver = Bounds::product(width.get(), evaluation.join_gap.upper().max(N::ZERO))
                    .map_err(|_| SolverError::ArithmeticFailure)?;
                let ideal = evaluation
                    .quote
                    .backing_change
                    .checked_add(
                        Bounds::new(N::ZERO, sliver.upper())
                            .map_err(|_| SolverError::ArithmeticFailure)?,
                    )
                    .map_err(|_| SolverError::ArithmeticFailure)?;
                if ideal.lower() > N::ZERO {
                    if end == max_ramp_end {
                        return Err(SolverError::NoSolution);
                    }
                    lo = end;
                } else if ideal.upper() < N::ZERO {
                    hi = end;
                } else {
                    return context.scan_ends(floor_increase_by, floor, lo, hi);
                }
            }
        }
        end = midpoint(lo, hi)?.ok_or(SolverError::PrecisionLimit)?;
    }
}

/// Find the area-preserving floor at the supplied ramp end.
///
/// In ideal arithmetic the highest area-preserving floor is nondecreasing with
/// ramp end, so clients can pass their maximum permitted end. The returned
/// floor is a conservative approximation under the supplied area/join limits,
/// not a claim of global optimality among all representable geometries.
pub fn solve_floor_at_ramp_end<N: SignedFixed>(
    calculator: &AvmCalculator<N>,
    state: &MarketState<N>,
    ramp_end: NonNeg<N>,
    search: FloorRaiseSearch<N>,
    engine: &dyn CharacteristicEngine<N>,
) -> Result<FloorRaiseSolution<N>> {
    let mut context = Search::new(calculator, state, ramp_end, search, engine)?;
    let ramp = context.ramp(ramp_end)?;
    let mut lo = state
        .segmentation()
        .ramp_start()
        .map_err(SolverError::Calculator)?;
    let mut hi = ramp_end;
    if let Some(solution) = context.try_start(&ramp, lo)? {
        return Ok(solution);
    }
    let mut start = hi;
    loop {
        let price = context.ramp_value(&ramp, start)?;
        let old_floor = state.segmentation().floor().get();
        if price.upper() <= old_floor {
            if start == ramp_end {
                return Err(SolverError::NoSolution);
            }
            lo = start;
        } else if price.lower() <= old_floor {
            return Err(SolverError::PrecisionLimit);
        } else {
            let increase = Pos::new(
                price
                    .lower()
                    .checked_sub(old_floor)
                    .map_err(|_| SolverError::ArithmeticFailure)?,
            )
            .map_err(|_| SolverError::ArithmeticFailure)?;
            let candidate = proposal(increase, ramp_end, start)?;
            let evaluation = context.evaluate(candidate)?;
            // The rounded-down floor removes only a rectangle from the ideal
            // continuous schedule. Include it in the root's residual bounds.
            let tail = evaluation
                .quote
                .next_state
                .floor_tail()
                .map_err(SolverError::Calculator)?;
            let rounding = price
                .upper()
                .checked_sub(price.lower())
                .map_err(|_| SolverError::ArithmeticFailure)?;
            let rectangle = Bounds::product(rounding, tail.get())
                .map_err(|_| SolverError::ArithmeticFailure)?;
            let ideal = evaluation
                .quote
                .backing_change
                .checked_add(
                    Bounds::new(N::ZERO, rectangle.upper())
                        .map_err(|_| SolverError::ArithmeticFailure)?,
                )
                .map_err(|_| SolverError::ArithmeticFailure)?;
            if let Some(candidate) = context.fit_floor(candidate, evaluation)? {
                return context.finish(candidate);
            }
            if ideal.upper() < N::ZERO {
                lo = start;
            } else if ideal.lower() > N::ZERO {
                hi = start;
            } else {
                return context.scan_starts(&ramp, lo, hi);
            }
        }
        start = midpoint(lo, hi)?.ok_or(SolverError::PrecisionLimit)?;
    }
}

struct Ramp<N: SignedFixed> {
    end: NonNeg<N>,
    at_end: Bounds<NonNeg<N>>,
}

struct Search<'a, N: SignedFixed> {
    calculator: &'a AvmCalculator<N>,
    state: &'a MarketState<N>,
    settings: FloorRaiseSearch<N>,
    remaining: u32,
    engine: &'a dyn CharacteristicEngine<N>,
}

impl<'a, N: SignedFixed> Search<'a, N> {
    fn new(
        calculator: &'a AvmCalculator<N>,
        state: &'a MarketState<N>,
        end: NonNeg<N>,
        settings: FloorRaiseSearch<N>,
        engine: &'a dyn CharacteristicEngine<N>,
    ) -> Result<Self> {
        if end < state.segmentation().ramp_end() || end > state.accounting().progress() {
            return Err(SolverError::InvalidRange);
        }
        Ok(Self {
            calculator,
            state,
            settings,
            remaining: settings.max_iterations.get(),
            engine,
        })
    }

    fn spend(&mut self) -> Result<()> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(SolverError::IterationLimit)?;
        Ok(())
    }

    fn point(&mut self, x: NonNeg<N>) -> Result<Bounds<NonNeg<N>>> {
        if x == NonNeg::zero() {
            return Ok(Bounds::exact(NonNeg::zero()));
        }
        self.spend()?;
        let [response] =
            evaluate_spans(self.engine, [point_request(x)]).map_err(SolverError::Calculator)?;
        Ok(response.y_at_lo)
    }

    fn ramp(&mut self, end: NonNeg<N>) -> Result<Ramp<N>> {
        Ok(Ramp {
            end,
            at_end: self.point(end)?,
        })
    }

    fn ramp_value(&mut self, ramp: &Ramp<N>, start: NonNeg<N>) -> Result<Bounds<N>> {
        let offset = self.state.segmentation().main_offset();
        if start == ramp.end {
            main_price(ramp.at_end, offset).map_err(SolverError::Calculator)
        } else {
            ramp_price(
                self.point(start)?,
                ramp.at_end,
                offset,
                self.calculator.ramp_scalar(),
            )
            .map_err(SolverError::Calculator)
        }
    }

    fn gap(&mut self, ramp: &Ramp<N>, start: NonNeg<N>, floor: NonNeg<N>) -> Result<Bounds<N>> {
        self.ramp_value(ramp, start)?
            .checked_sub_exact(floor.get())
            .map_err(|_| SolverError::ArithmeticFailure)
    }

    fn intersection(
        &mut self,
        ramp: &Ramp<N>,
        floor: NonNeg<N>,
    ) -> Result<Option<Bounds<NonNeg<N>>>> {
        let mut lo = self
            .state
            .segmentation()
            .ramp_start()
            .map_err(SolverError::Calculator)?;
        let mut hi = ramp.end;
        let low_gap = self.gap(ramp, lo, floor)?;
        let high_gap = self.gap(ramp, hi, floor)?;
        if low_gap.lower() > N::ZERO || high_gap.upper() < N::ZERO {
            return Ok(None);
        }
        if low_gap.is_exact() && low_gap.lower() == N::ZERO {
            return Ok(Some(Bounds::exact(lo)));
        }
        if high_gap.is_exact() && high_gap.lower() == N::ZERO {
            return Ok(Some(Bounds::exact(hi)));
        }
        if low_gap.upper() > N::ZERO || high_gap.lower() < N::ZERO {
            return Err(SolverError::PrecisionLimit);
        }
        while let Some(mid) = midpoint(lo, hi)? {
            let gap = self.gap(ramp, mid, floor)?;
            if gap.is_exact() && gap.lower() == N::ZERO {
                return Ok(Some(Bounds::exact(mid)));
            }
            if gap.upper() <= N::ZERO {
                lo = mid;
            } else if gap.lower() >= N::ZERO {
                hi = mid;
            } else {
                // An uncertain point cannot select a side. Its neighbours can
                // sometimes certify a much narrower bracket; otherwise retain
                // the existing certified bracket for the outer area check.
                let quantum =
                    NonNeg::new(N::QUANTUM).map_err(|_| SolverError::ArithmeticFailure)?;
                let left = mid
                    .checked_sub(quantum)
                    .map_err(|_| SolverError::ArithmeticFailure)?;
                let right = mid
                    .checked_add(quantum)
                    .map_err(|_| SolverError::ArithmeticFailure)?;
                if left > lo {
                    let gap = self.gap(ramp, left, floor)?;
                    if gap.is_exact() && gap.lower() == N::ZERO {
                        return Ok(Some(Bounds::exact(left)));
                    }
                    if gap.upper() <= N::ZERO {
                        lo = left;
                    }
                }
                if right < hi {
                    let gap = self.gap(ramp, right, floor)?;
                    if gap.is_exact() && gap.lower() == N::ZERO {
                        return Ok(Some(Bounds::exact(right)));
                    }
                    if gap.lower() >= N::ZERO {
                        hi = right;
                    }
                }
                break;
            }
        }
        Ok(Some(
            Bounds::new(lo, hi).map_err(|_| SolverError::ArithmeticFailure)?,
        ))
    }

    fn evaluate(&mut self, proposal: FloorRaiseProposal<N>) -> Result<FloorRaiseEvaluation<N>> {
        self.spend()?;
        self.calculator
            .evaluate_floor_raise(self.state, proposal, self.engine)
            .map_err(SolverError::Calculator)
    }

    /// Near a root, interval uncertainty can hide an exactly representable
    /// answer. Check the remaining grid points without guessing its direction.
    /// Every point still consumes the shared characteristic-call budget.
    fn scan_ends(
        &mut self,
        increase: Pos<N>,
        floor: NonNeg<N>,
        lo: NonNeg<N>,
        hi: NonNeg<N>,
    ) -> Result<FloorRaiseSolution<N>> {
        scan_points(lo, hi, |end| {
            let ramp = self.ramp(end)?;
            match self.intersection(&ramp, floor) {
                Ok(Some(intersection)) => {
                    let candidate = proposal(increase, end, intersection.upper())?;
                    let evaluation = self.evaluate(candidate)?;
                    if self.accepts(&evaluation)? {
                        return self.finish(candidate).map(Some);
                    }
                }
                Ok(None) | Err(SolverError::PrecisionLimit) => {}
                Err(error) => return Err(error),
            }
            Ok(None)
        })
    }

    fn scan_starts(
        &mut self,
        ramp: &Ramp<N>,
        lo: NonNeg<N>,
        hi: NonNeg<N>,
    ) -> Result<FloorRaiseSolution<N>> {
        scan_points(lo, hi, |start| self.try_start(ramp, start))
    }

    fn try_start(
        &mut self,
        ramp: &Ramp<N>,
        start: NonNeg<N>,
    ) -> Result<Option<FloorRaiseSolution<N>>> {
        let price = self.ramp_value(ramp, start)?;
        let old_floor = self.state.segmentation().floor().get();
        if price.lower() <= old_floor {
            return Ok(None);
        }
        let increase = Pos::new(
            price
                .lower()
                .checked_sub(old_floor)
                .map_err(|_| SolverError::ArithmeticFailure)?,
        )
        .map_err(|_| SolverError::ArithmeticFailure)?;
        let candidate = proposal(increase, ramp.end, start)?;
        let evaluation = self.evaluate(candidate)?;
        match self.fit_floor(candidate, evaluation)? {
            Some(candidate) => self.finish(candidate).map(Some),
            None => Ok(None),
        }
    }

    fn accepts(&self, evaluation: &FloorRaiseEvaluation<N>) -> Result<bool> {
        let minimum = self
            .settings
            .max_area_decrease
            .get()
            .checked_neg()
            .map_err(|_| SolverError::ArithmeticFailure)?;
        Ok(evaluation.join_gap.lower() >= N::ZERO
            && evaluation.join_gap.upper() <= self.settings.max_floor_join_gap.get()
            && evaluation.quote.backing_change.lower() >= minimum
            && evaluation.quote.backing_change.upper() <= N::ZERO)
    }

    fn fit_floor(
        &mut self,
        proposal: FloorRaiseProposal<N>,
        evaluation: FloorRaiseEvaluation<N>,
    ) -> Result<Option<FloorRaiseProposal<N>>> {
        if self.accepts(&evaluation)? {
            return Ok(Some(proposal));
        }
        let excess = evaluation.quote.backing_change.upper();
        let tail = evaluation
            .quote
            .next_state
            .floor_tail()
            .map_err(SolverError::Calculator)?;
        if excess <= N::ZERO || tail == NonNeg::zero() {
            return Ok(None);
        }
        let decrease = excess
            .checked_div_ceil(tail.get())
            .map_err(|_| SolverError::ArithmeticFailure)?;
        let gap = evaluation
            .join_gap
            .upper()
            .checked_add(decrease)
            .map_err(|_| SolverError::ArithmeticFailure)?;
        if gap > self.settings.max_floor_join_gap.get()
            || decrease >= proposal.floor_increase_by.get()
        {
            return Ok(None);
        }
        let proposal = FloorRaiseProposal {
            floor_increase_by: Pos::new(
                proposal
                    .floor_increase_by
                    .get()
                    .checked_sub(decrease)
                    .map_err(|_| SolverError::ArithmeticFailure)?,
            )
            .map_err(|_| SolverError::ArithmeticFailure)?,
            ..proposal
        };
        let evaluation = self.evaluate(proposal)?;
        Ok(self.accepts(&evaluation)?.then_some(proposal))
    }

    fn finish(&mut self, proposal: FloorRaiseProposal<N>) -> Result<FloorRaiseSolution<N>> {
        let allowance = self
            .settings
            .max_area_decrease
            .get()
            .checked_neg()
            .map_err(|_| SolverError::ArithmeticFailure)?;
        self.spend()?;
        let quote = self
            .calculator
            .verify_floor_raise(
                self.state,
                proposal,
                FloorRaisePolicy::new(self.settings.max_floor_join_gap, allowance),
                self.engine,
            )
            .map_err(SolverError::Calculator)?;
        Ok(FloorRaiseSolution { proposal, quote })
    }
}

fn proposal<N: SignedFixed>(
    floor_increase_by: Pos<N>,
    end: NonNeg<N>,
    start: NonNeg<N>,
) -> Result<FloorRaiseProposal<N>> {
    Ok(FloorRaiseProposal {
        floor_increase_by,
        ramp_end: end,
        ramp_width: end
            .checked_sub(start)
            .map_err(|_| SolverError::ArithmeticFailure)?,
    })
}

fn midpoint<N: SignedFixed>(lo: NonNeg<N>, hi: NonNeg<N>) -> Result<Option<NonNeg<N>>> {
    let width = hi
        .checked_sub(lo)
        .map_err(|_| SolverError::ArithmeticFailure)?;
    let two = N::from_i32(2).map_err(|_| SolverError::ArithmeticFailure)?;
    let half = width
        .get()
        .checked_div_floor(two)
        .map_err(|_| SolverError::ArithmeticFailure)?;
    let mid = NonNeg::new(
        lo.get()
            .checked_add(half)
            .map_err(|_| SolverError::ArithmeticFailure)?,
    )
    .map_err(|_| SolverError::ArithmeticFailure)?;
    Ok((mid > lo && mid < hi).then_some(mid))
}

fn scan_points<N: SignedFixed, T>(
    lo: NonNeg<N>,
    hi: NonNeg<N>,
    mut evaluate: impl FnMut(NonNeg<N>) -> Result<Option<T>>,
) -> Result<T> {
    let quantum = NonNeg::new(N::QUANTUM).map_err(|_| SolverError::ArithmeticFailure)?;
    let mut point = lo;
    while point < hi {
        point = point
            .checked_add(quantum)
            .map_err(|_| SolverError::ArithmeticFailure)?;
        if point == hi {
            break;
        }
        if let Some(solution) = evaluate(point)? {
            return Ok(solution);
        }
    }
    Err(SolverError::PrecisionLimit)
}

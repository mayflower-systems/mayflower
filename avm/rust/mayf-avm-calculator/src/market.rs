//! Shared market arithmetic, independent of the characteristic implementation.

use mayf_fixed_number::{AboveOne, Bounds, NonNeg, Pos, SignedFixed};

use crate::calculator::{
    ScheduleCalculator, add_bounds, constant_area, evaluate_optional_spans, main_area, main_price,
    ramp_area, ramp_price, span_between,
};
use crate::{
    Accounting, CharacteristicEngine, Error, MarketState, Result, Segmentation, SpanRequest,
};

/// A reserve quote and the state to persist after successful settlement.
/// Charge the upper reserve bound for issuance; pay the lower for redemption.
/// Redemption reserves any extra backing caused by rounding a contracted main
/// offset upward. Normalization, whole-token rounding, fees, and custody belong
/// to the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TradeQuote<N: SignedFixed> {
    pub next_state: MarketState<N>,
    pub reserve: Bounds<NonNeg<N>>,
}

/// Untrusted proposed geometry. Verification checks its shape and transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloorRaiseProposal<N: SignedFixed> {
    /// Positive change from the current floor; a decrease is unrepresentable.
    pub floor_increase_by: Pos<N>,
    pub ramp_end: NonNeg<N>,
    pub ramp_width: NonNeg<N>,
}

/// Acceptance limits in normalized floor-join price and reserve units.
///
/// The ramp may start at or above the floor, with the full upward gap enclosed
/// by `max_floor_join_gap`. Main placement is preserved exactly.
///
/// The signed area allowance specifies both direction and maximum magnitude:
/// positive permits backing growth in `[0, allowance]`, negative permits a
/// decrease in `[allowance, 0]`, and zero requires unchanged backing. It is not
/// an absolute-difference tolerance. The full calculated enclosure must fit.
///
/// A higher-level program selects the allowance for approximate preservation
/// or growth from reserve surplus, and independently establishes funding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloorRaisePolicy<N: SignedFixed> {
    max_floor_join_gap: NonNeg<N>,
    area_change_tolerance: N,
}

impl<N: SignedFixed> FloorRaisePolicy<N> {
    pub const fn new(max_floor_join_gap: NonNeg<N>, area_change_tolerance: N) -> Self {
        Self {
            max_floor_join_gap,
            area_change_tolerance,
        }
    }

    pub fn exact() -> Self {
        Self::new(NonNeg::zero(), N::ZERO)
    }

    pub const fn max_floor_join_gap(&self) -> NonNeg<N> {
        self.max_floor_join_gap
    }

    pub const fn area_change_tolerance(&self) -> N {
        self.area_change_tolerance
    }
}

/// An accepted proposal and its measured changes, candidate minus previous.
/// These are arithmetic results, not evidence of available reserve funds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloorRaiseQuote<N: SignedFixed> {
    pub next_state: MarketState<N>,
    /// Total proposed backing, including the independent physical floor tail.
    pub backing: Bounds<NonNeg<N>>,
    pub backing_change: Bounds<N>,
}

/// Numerical evidence for a candidate, before its acceptance policy is checked.
pub(crate) struct FloorRaiseEvaluation<N: SignedFixed> {
    pub quote: FloorRaiseQuote<N>,
    pub join_gap: Bounds<N>,
}

/// Stateless AVM arithmetic with an immutable ramp scalar.
///
/// The caller supplies state snapshots and a numerical characteristic engine.
/// Each operation plans all required spans before invoking that engine at most
/// once. The engine receives no segmentation, accounting, or policy data.
/// Explicit floor issuance and redemption never invoke it.
///
/// A program wrapper owns number selection, authorization, funding, fees, and
/// persistence. It must bind a market to the same characteristic parameters
/// and ramp scalar throughout its lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvmCalculator<N: SignedFixed> {
    ramp_scalar: AboveOne<N>,
}

impl<N: SignedFixed> AvmCalculator<N> {
    pub const fn new(ramp_scalar: AboveOne<N>) -> Self {
        Self { ramp_scalar }
    }

    pub const fn ramp_scalar(&self) -> AboveOne<N> {
        self.ramp_scalar
    }

    fn schedule(&self, state: &MarketState<N>) -> ScheduleCalculator<N> {
        ScheduleCalculator::new(state.segmentation(), self.ramp_scalar)
    }

    pub fn price(
        &self,
        state: &MarketState<N>,
        characteristic: &dyn CharacteristicEngine<N>,
    ) -> Result<Bounds<NonNeg<N>>> {
        self.schedule(state)
            .y(state.accounting().progress(), characteristic)
    }

    /// Required backing, including the physical floor tail when circulation
    /// differs from progress. This is independent of actual reserve custody.
    pub fn backing(
        &self,
        state: &MarketState<N>,
        characteristic: &dyn CharacteristicEngine<N>,
    ) -> Result<Bounds<NonNeg<N>>> {
        let floor_area = constant_area(state.segmentation().floor(), state.floor_tail()?)?;
        let start = state.segmentation().ramp_start()?;
        let width = state
            .accounting()
            .progress()
            .checked_sub(start)
            .map_err(|_| Error::ArithmeticFailure)?;
        let nonfloor_area = self
            .schedule(state)
            .integrate(start, width, characteristic)?;
        add_bounds(floor_area, nonfloor_area)
    }

    pub fn quote_issue(
        &self,
        state: &MarketState<N>,
        amount: Pos<N>,
        characteristic: &dyn CharacteristicEngine<N>,
    ) -> Result<TradeQuote<N>> {
        let accounting = state.accounting();
        let progress = accounting
            .progress()
            .checked_add(amount)
            .map_err(|_| Error::ArithmeticFailure)?;
        let circulating_supply = accounting
            .circulating_supply()
            .checked_add(amount)
            .map_err(|_| Error::ArithmeticFailure)?;
        let next_state = MarketState::new(
            state.segmentation(),
            Accounting::new(progress, circulating_supply),
        )?;
        let reserve =
            self.schedule(state)
                .integrate(accounting.progress(), amount.into(), characteristic)?;
        Ok(TradeQuote {
            next_state,
            reserve,
        })
    }

    pub fn quote_redeem(
        &self,
        state: &MarketState<N>,
        amount: Pos<N>,
        characteristic: &dyn CharacteristicEngine<N>,
    ) -> Result<TradeQuote<N>> {
        let amount = NonNeg::from(amount);
        let accounting = state.accounting();
        if amount > accounting.circulating_supply() {
            return Err(Error::InsufficientCirculation);
        }
        let on_schedule = amount.min(accounting.progress());
        let remaining = accounting
            .progress()
            .checked_sub(on_schedule)
            .map_err(|_| Error::ArithmeticFailure)?;
        let at_floor = amount
            .checked_sub(on_schedule)
            .map_err(|_| Error::ArithmeticFailure)?;
        let circulating_supply = accounting
            .circulating_supply()
            .checked_sub(amount)
            .map_err(|_| Error::ArithmeticFailure)?;
        let (on_schedule_reserve, segmentation) =
            self.schedule(state)
                .redeem(remaining, accounting.progress(), characteristic)?;
        let next_state =
            MarketState::new(segmentation, Accounting::new(remaining, circulating_supply))?;
        let reserve = add_bounds(
            constant_area(state.segmentation().floor(), at_floor)?,
            on_schedule_reserve,
        )?;
        Ok(TradeQuote {
            next_state,
            reserve,
        })
    }

    /// Add circulation at the floor without moving characteristic progress.
    /// A program can use this arithmetic for option exercise after authorization.
    pub fn quote_issue_at_floor(
        &self,
        state: &MarketState<N>,
        amount: Pos<N>,
    ) -> Result<TradeQuote<N>> {
        let accounting = state.accounting();
        let circulating_supply = accounting
            .circulating_supply()
            .checked_add(amount)
            .map_err(|_| Error::ArithmeticFailure)?;
        let next_state = MarketState::new(
            state.segmentation(),
            Accounting::new(accounting.progress(), circulating_supply),
        )?;
        let reserve = constant_area(state.segmentation().floor(), amount.into())?;
        Ok(TradeQuote {
            next_state,
            reserve,
        })
    }

    /// Remove physical floor-tail supply without changing progress or geometry.
    pub fn quote_redeem_at_floor(
        &self,
        state: &MarketState<N>,
        amount: Pos<N>,
    ) -> Result<TradeQuote<N>> {
        let amount = NonNeg::from(amount);
        if amount > state.floor_tail()? {
            return Err(Error::InsufficientFloorTail);
        }
        let accounting = state.accounting();
        let circulating_supply = accounting
            .circulating_supply()
            .checked_sub(amount)
            .map_err(|_| Error::ArithmeticFailure)?;
        let next_state = MarketState::new(
            state.segmentation(),
            Accounting::new(accounting.progress(), circulating_supply),
        )?;
        let reserve = constant_area(state.segmentation().floor(), amount)?;
        Ok(TradeQuote {
            next_state,
            reserve,
        })
    }

    /// Verify a floor increase without changing main placement. The changed area
    /// stops at the new ramp end; the unchanged main tail is evaluated only for
    /// the total backing needed by a caller's reserve-funding check.
    pub fn verify_floor_raise(
        &self,
        state: &MarketState<N>,
        proposal: FloorRaiseProposal<N>,
        policy: FloorRaisePolicy<N>,
        characteristic: &dyn CharacteristicEngine<N>,
    ) -> Result<FloorRaiseQuote<N>> {
        let evaluation = self.evaluate_floor_raise(state, proposal, characteristic)?;
        if evaluation.join_gap.lower() < N::ZERO
            || evaluation.join_gap.upper() > policy.max_floor_join_gap.get()
        {
            return Err(Error::FloorJoinOutsideTolerance);
        }
        let allowed_lower = policy.area_change_tolerance.min(N::ZERO);
        let allowed_upper = policy.area_change_tolerance.max(N::ZERO);
        if evaluation.quote.backing_change.lower() < allowed_lower
            || evaluation.quote.backing_change.upper() > allowed_upper
        {
            return Err(Error::AreaChangeOutsideTolerance);
        }
        Ok(evaluation.quote)
    }

    pub(crate) fn evaluate_floor_raise(
        &self,
        state: &MarketState<N>,
        proposal: FloorRaiseProposal<N>,
        characteristic: &dyn CharacteristicEngine<N>,
    ) -> Result<FloorRaiseEvaluation<N>> {
        let old = state.segmentation();
        let floor = old
            .floor()
            .checked_add(proposal.floor_increase_by)
            .map_err(|_| Error::ArithmeticFailure)?;
        let candidate = Segmentation::new(
            floor,
            proposal.ramp_end,
            proposal.ramp_width,
            old.main_offset(),
        )?;
        if candidate.ramp_end() < old.ramp_end() || candidate.ramp_start()? < old.ramp_start()? {
            return Err(Error::RampMovedBackward);
        }
        let next_state = MarketState::new(candidate, state.accounting())?;
        let old_ramp_request = span_between(old.ramp_start()?, old.ramp_end())?;
        // The new ramp also supplies its endpoint values for the join check,
        // including when its width is zero.
        let new_ramp_request = SpanRequest {
            lo: candidate.ramp_start()?,
            width: candidate.ramp_width(),
        };
        let bridge_request = span_between(old.ramp_end(), candidate.ramp_end())?;
        let tail_request = span_between(candidate.ramp_end(), state.accounting().progress())?;
        let [old_ramp, new_ramp, bridge, tail] = evaluate_optional_spans(
            characteristic,
            [
                old_ramp_request,
                Some(new_ramp_request),
                bridge_request,
                tail_request,
            ],
        )?;
        let new_ramp = new_ramp.ok_or(Error::InvalidCharacteristicBatch)?;
        let at_start = if candidate.ramp_width() == NonNeg::zero() {
            main_price(new_ramp.y_at_lo, candidate.main_offset())?
        } else {
            ramp_price(
                new_ramp.y_at_lo,
                new_ramp.y_at_hi,
                candidate.main_offset(),
                self.ramp_scalar,
            )?
        };
        let join_gap = at_start
            .checked_sub_exact(floor.get())
            .map_err(|_| Error::ArithmeticFailure)?;

        let mut old_prefix = constant_area(old.floor(), state.floor_tail()?)?.into_raw();
        if let (Some(request), Some(response)) = (old_ramp_request, old_ramp) {
            old_prefix = old_prefix
                .checked_add(ramp_area(
                    response,
                    request.width,
                    response.y_at_hi,
                    old.main_offset(),
                    self.ramp_scalar,
                )?)
                .map_err(|_| Error::ArithmeticFailure)?;
        }
        if let (Some(request), Some(response)) = (bridge_request, bridge) {
            old_prefix = old_prefix
                .checked_add(main_area(response, request.width, old.main_offset())?)
                .map_err(|_| Error::ArithmeticFailure)?;
        }
        let mut new_prefix = constant_area(floor, next_state.floor_tail()?)?.into_raw();
        if candidate.ramp_width() != NonNeg::zero() {
            new_prefix = new_prefix
                .checked_add(ramp_area(
                    new_ramp,
                    candidate.ramp_width(),
                    new_ramp.y_at_hi,
                    candidate.main_offset(),
                    self.ramp_scalar,
                )?)
                .map_err(|_| Error::ArithmeticFailure)?;
        }
        let backing_change = new_prefix
            .checked_sub(old_prefix)
            .map_err(|_| Error::ArithmeticFailure)?;
        let backing = if let (Some(request), Some(response)) = (tail_request, tail) {
            new_prefix
                .checked_add(main_area(response, request.width, candidate.main_offset())?)
                .map_err(|_| Error::ArithmeticFailure)?
        } else {
            new_prefix
        };
        Ok(FloorRaiseEvaluation {
            quote: FloorRaiseQuote {
                next_state,
                backing_change,
                backing: backing
                    .try_nonnegative()
                    .map_err(|_| Error::NegativeOutput)?,
            },
            join_gap,
        })
    }
}

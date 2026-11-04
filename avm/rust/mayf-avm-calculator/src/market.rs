//! Shared market arithmetic, independent of the characteristic implementation.

use mayf_fixed_number::{AboveOne, Bounds, NonNeg, Pos, SignedFixed};

use crate::calculator::{
    ScheduleCalculator, add_bounds, constant_area, evaluate_spans, floor_enclosure, scale,
};
use crate::{
    Accounting, CharacteristicEngine, Error, MarketState, Result, Segmentation, SpanRequest,
    SpanResponse,
};

/// A gross reserve quote and the state to persist after successful settlement.
/// Charge the upper reserve bound for issuance; pay the lower for redemption.
/// Normalization, whole-token rounding, fees, and custody belong to the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TradeQuote<N: SignedFixed> {
    pub next_state: MarketState<N>,
    pub reserve: Bounds<NonNeg<N>>,
}

/// Untrusted proposed geometry. Verification checks its shape and transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloorRaiseProposal<N: SignedFixed> {
    pub floor: NonNeg<N>,
    pub ramp_end: NonNeg<N>,
    pub ramp_width: NonNeg<N>,
}

/// Acceptance limits in normalized price and reserve units.
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
    max_price_change: NonNeg<N>,
    area_change_tolerance: N,
}

impl<N: SignedFixed> FloorRaisePolicy<N> {
    pub const fn new(max_price_change: NonNeg<N>, area_change_tolerance: N) -> Self {
        Self {
            max_price_change,
            area_change_tolerance,
        }
    }

    pub fn exact() -> Self {
        Self::new(NonNeg::zero(), N::ZERO)
    }

    pub const fn max_price_change(&self) -> NonNeg<N> {
        self.max_price_change
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
    pub main_change: Bounds<N>,
    pub backing_change: Bounds<N>,
}

/// Stateless AVM arithmetic with an immutable ramp scalar.
///
/// The caller supplies state snapshots and a numerical characteristic engine.
/// Each operation plans all required spans before invoking that engine at most
/// once. The engine receives no segmentation, accounting, or policy data.
/// Floor-only operations never invoke it.
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
        let next_state = MarketState::new(
            state.segmentation().contract_to(remaining)?,
            Accounting::new(remaining, circulating_supply),
        )?;

        // Always quote the old schedule. Contraction preserves its retained
        // prefix but changes the main segment above the remaining progress.
        let reserve = add_bounds(
            constant_area(state.segmentation().floor(), at_floor)?,
            self.schedule(state)
                .integrate(remaining, on_schedule, characteristic)?,
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

    /// Verify proposed geometry using one numerical batch, without solving for
    /// any parameter. Both main placement and backing are compared centrally.
    /// Valid raises can narrow the ramp; width is not a monotonic invariant.
    pub fn verify_floor_raise(
        &self,
        state: &MarketState<N>,
        proposal: FloorRaiseProposal<N>,
        policy: FloorRaisePolicy<N>,
        characteristic: &dyn CharacteristicEngine<N>,
    ) -> Result<FloorRaiseQuote<N>> {
        let old = state.segmentation();
        if proposal.floor <= old.floor() {
            return Err(Error::FloorNotRaised);
        }
        let candidate = Segmentation::new(proposal.floor, proposal.ramp_end, proposal.ramp_width)?;
        if candidate.ramp_end() < old.ramp_end() || candidate.ramp_start()? < old.ramp_start()? {
            return Err(Error::RampMovedBackward);
        }
        let next_state = MarketState::new(candidate, state.accounting())?;
        let progress = state.accounting().progress();
        let start_movement = SpanRequest {
            lo: old.ramp_start()?,
            width: candidate
                .ramp_start()?
                .checked_sub(old.ramp_start()?)
                .map_err(|_| Error::ArithmeticFailure)?,
        };
        let end_movement = SpanRequest {
            lo: old.ramp_end(),
            width: candidate
                .ramp_end()
                .checked_sub(old.ramp_end())
                .map_err(|_| Error::ArithmeticFailure)?,
        };
        let (start, end) =
            evaluate_anchor_changes(characteristic, progress, start_movement, end_movement)?;
        let floor_increase = candidate
            .floor()
            .checked_sub(old.floor())
            .map_err(|_| Error::ArithmeticFailure)?;
        let floor_backing =
            constant_area(floor_increase, state.accounting().circulating_supply())?.into_raw();

        // Moving an anchor removes area from its vertically anchored tail.
        // The main's shared neutral integral cancels, leaving only these two
        // boundary movements. The formula includes any permitted main change.
        let (main_change, backing_change) = if start_movement == end_movement {
            // Both ramps are collapsed, so their identical contributions have
            // net coefficient -1. Reuse the result without widening its bounds.
            (
                Bounds::exact(floor_increase.get())
                    .checked_sub(start.height)
                    .map_err(|_| Error::ArithmeticFailure)?,
                floor_backing
                    .checked_sub(start.area)
                    .map_err(|_| Error::ArithmeticFailure)?,
            )
        } else {
            let ramp_gain = NonNeg::from(self.ramp_scalar);
            let main_gain = ramp_gain
                .checked_sub(NonNeg::one())
                .map_err(|_| Error::ArithmeticFailure)?;
            let main_change = Bounds::exact(floor_increase.get())
                .checked_add(scale(end.height, main_gain.get())?)
                .map_err(|_| Error::ArithmeticFailure)?
                .checked_sub(scale(start.height, ramp_gain.get())?)
                .map_err(|_| Error::ArithmeticFailure)?;
            let backing_change = floor_backing
                .checked_add(scale(end.area, main_gain.get())?)
                .map_err(|_| Error::ArithmeticFailure)?
                .checked_sub(scale(start.area, ramp_gain.get())?)
                .map_err(|_| Error::ArithmeticFailure)?;
            (main_change, backing_change)
        };
        let minimum_price_change = policy
            .max_price_change
            .get()
            .checked_neg()
            .map_err(|_| Error::ArithmeticFailure)?;
        if main_change.lower() < minimum_price_change
            || main_change.upper() > policy.max_price_change.get()
        {
            return Err(Error::MainCurveChanged);
        }

        let allowed_lower = policy.area_change_tolerance.min(N::ZERO);
        let allowed_upper = policy.area_change_tolerance.max(N::ZERO);
        if backing_change.lower() < allowed_lower || backing_change.upper() > allowed_upper {
            return Err(Error::AreaChangeOutsideTolerance);
        }
        Ok(FloorRaiseQuote {
            next_state,
            main_change,
            backing_change,
        })
    }
}

/// Effect of moving the origin of `f(x) - f(anchor)` forward.
#[derive(Clone, Copy)]
struct AnchorChange<N: SignedFixed> {
    height: Bounds<N>,
    area: Bounds<N>,
}

impl<N: SignedFixed> AnchorChange<N> {
    fn zero() -> Self {
        Self {
            height: Bounds::exact(N::ZERO),
            area: Bounds::exact(N::ZERO),
        }
    }
}

fn evaluate_anchor_changes<N: SignedFixed>(
    characteristic: &dyn CharacteristicEngine<N>,
    progress: NonNeg<N>,
    start: SpanRequest<N>,
    end: SpanRequest<N>,
) -> Result<(AnchorChange<N>, AnchorChange<N>)> {
    let zero = AnchorChange::zero();
    match (start.width != NonNeg::zero(), end.width != NonNeg::zero()) {
        (false, false) => Ok((zero, zero)),
        (true, false) => {
            let [response] = evaluate_spans(characteristic, [start])?;
            Ok((anchor_change(start, response, progress)?, zero))
        }
        (false, true) => {
            let [response] = evaluate_spans(characteristic, [end])?;
            Ok((zero, anchor_change(end, response, progress)?))
        }
        (true, true) if start == end => {
            let [response] = evaluate_spans(characteristic, [start])?;
            let change = anchor_change(start, response, progress)?;
            Ok((change, change))
        }
        (true, true) => {
            let [start_response, end_response] = evaluate_spans(characteristic, [start, end])?;
            Ok((
                anchor_change(start, start_response, progress)?,
                anchor_change(end, end_response, progress)?,
            ))
        }
    }
}

fn anchor_change<N: SignedFixed>(
    request: SpanRequest<N>,
    response: SpanResponse<N>,
    progress: NonNeg<N>,
) -> Result<AnchorChange<N>> {
    let hi = request
        .lo
        .checked_add(request.width)
        .map_err(|_| Error::ArithmeticFailure)?;
    let remaining = progress
        .checked_sub(hi)
        .map_err(|_| Error::ArithmeticFailure)?;
    // Monotonicity proves both the height increase and the area above f(lo)
    // are nonnegative, even if independent endpoint bounds straddle zero.
    let height = response
        .y_at_hi
        .into_raw()
        .checked_sub(response.y_at_lo.into_raw())
        .map_err(|_| Error::ArithmeticFailure)?;
    let height = floor_enclosure(height, NonNeg::zero())?;
    let rectangle = scale(response.y_at_lo.into_raw(), request.width.get())?;
    let span_area = response
        .area
        .into_raw()
        .checked_sub(rectangle)
        .map_err(|_| Error::ArithmeticFailure)?;
    let span_area = floor_enclosure(span_area, NonNeg::zero())?;
    // G(lo, hi) = integral(lo, hi-lo) - f(lo)*(hi-lo)
    //             + (progress-hi)*(f(hi)-f(lo)).
    let area = span_area
        .checked_add(scale(height, remaining.get())?)
        .map_err(|_| Error::ArithmeticFailure)?;
    Ok(AnchorChange { height, area })
}

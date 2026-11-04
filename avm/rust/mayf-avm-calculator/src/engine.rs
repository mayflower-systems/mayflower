use mayf_characteristic::Characteristic;
use mayf_fixed_number::{AboveOne, Bounds, NonNeg, Pos, SignedFixed};
use mayf_monotone::MonotoneCalculator;

use crate::calculator::{ScheduleCalculator, add_bounds, compile_schedule, constant_area};
use crate::error::arithmetic;
use crate::{Accounting, Error, MarketState, Result, Segmentation};

/// A gross reserve quote and the state to persist after successful settlement.
/// Charge the upper reserve bound for issuance; pay the lower for redemption.
/// Normalization, whole-token rounding, fees, and custody belong to the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TradeQuote<N: SignedFixed> {
    pub next_state: MarketState<N>,
    pub reserve: Bounds<NonNeg<N>>,
}

/// Untrusted proposed geometry. Verification checks both its shape and transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloorRaiseProposal<N: SignedFixed> {
    pub floor: Pos<N>,
    pub ramp_end: NonNeg<N>,
    pub ramp_width: NonNeg<N>,
}

/// Absolute limits in normalized price and reserve units, chosen by the host.
/// A raise may retain bounded reserve surplus, but cannot increase required backing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloorRaisePolicy<N: SignedFixed> {
    max_price_change: NonNeg<N>,
    max_reserve_surplus: NonNeg<N>,
}

impl<N: SignedFixed> FloorRaisePolicy<N> {
    pub const fn new(max_price_change: NonNeg<N>, max_reserve_surplus: NonNeg<N>) -> Self {
        Self {
            max_price_change,
            max_reserve_surplus,
        }
    }

    pub fn exact() -> Self {
        Self::new(NonNeg::zero(), NonNeg::zero())
    }

    pub const fn max_price_change(&self) -> NonNeg<N> {
        self.max_price_change
    }

    pub const fn max_reserve_surplus(&self) -> NonNeg<N> {
        self.max_reserve_surplus
    }
}

/// An accepted raise. Residuals are candidate minus previous schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloorRaiseQuote<N: SignedFixed> {
    pub next_state: MarketState<N>,
    pub main_change: Bounds<N>,
    pub backing_change: Bounds<N>,
}

/// Stateless quoting contract, independent of characteristic implementation.
///
/// The host supplies snapshots and persists receipts only after settlement.
/// It must bind each market to the same characteristic parameters and ramp scalar,
/// and choose floor-raise limits from trusted configuration. All amounts are in
/// normalized units; the wire format and token normalization are host concerns.
pub trait AvmCalculator<N: SignedFixed> {
    fn price(&self, state: &MarketState<N>) -> Result<Bounds<NonNeg<N>>>;
    fn backing(&self, state: &MarketState<N>) -> Result<Bounds<NonNeg<N>>>;

    fn quote_issue(&self, state: &MarketState<N>, amount: Pos<N>) -> Result<TradeQuote<N>>;
    fn quote_redeem(&self, state: &MarketState<N>, amount: Pos<N>) -> Result<TradeQuote<N>>;

    fn quote_issue_at_floor(&self, state: &MarketState<N>, amount: Pos<N>)
    -> Result<TradeQuote<N>>;
    fn quote_redeem_at_floor(
        &self,
        state: &MarketState<N>,
        amount: Pos<N>,
    ) -> Result<TradeQuote<N>>;

    fn verify_floor_raise(
        &self,
        state: &MarketState<N>,
        proposal: FloorRaiseProposal<N>,
        policy: FloorRaisePolicy<N>,
    ) -> Result<FloorRaiseQuote<N>>;
}

/// Gives any affine characteristic the complete shared AVM quoting contract.
/// Stores immutable engine configuration, never mutable market state.
pub struct CharacteristicEngine<C, N: SignedFixed> {
    characteristic: C,
    ramp_scalar: AboveOne<N>,
}

impl<C, N: SignedFixed> CharacteristicEngine<C, N> {
    pub const fn new(characteristic: C, ramp_scalar: AboveOne<N>) -> Self {
        Self {
            characteristic,
            ramp_scalar,
        }
    }
}

impl<C: Characteristic<N>, N: SignedFixed> CharacteristicEngine<C, N> {
    fn compile(&self, state: &MarketState<N>) -> Result<ScheduleCalculator<N, C::Calculator>> {
        compile_schedule(&self.characteristic, self.ramp_scalar, state.segmentation())
    }
}

impl<C: Characteristic<N>, N: SignedFixed> AvmCalculator<N> for CharacteristicEngine<C, N> {
    fn price(&self, state: &MarketState<N>) -> Result<Bounds<NonNeg<N>>> {
        self.compile(state)?.y(state.accounting().progress())
    }

    fn backing(&self, state: &MarketState<N>) -> Result<Bounds<NonNeg<N>>> {
        backing(&self.compile(state)?, state)
    }

    fn quote_issue(&self, state: &MarketState<N>, amount: Pos<N>) -> Result<TradeQuote<N>> {
        let accounting = state.accounting();
        let next_state = MarketState::new(
            state.segmentation(),
            Accounting::new(
                arithmetic(accounting.progress().checked_add(amount))?,
                arithmetic(accounting.circulating_supply().checked_add(amount))?,
            ),
        )?;
        let reserve = self
            .compile(state)?
            .integrate(accounting.progress(), amount.into())?;
        Ok(TradeQuote {
            next_state,
            reserve,
        })
    }

    fn quote_redeem(&self, state: &MarketState<N>, amount: Pos<N>) -> Result<TradeQuote<N>> {
        let amount = NonNeg::from(amount);
        let accounting = state.accounting();
        if amount > accounting.circulating_supply() {
            return Err(Error::InsufficientCirculation);
        }
        let on_schedule = amount.min(accounting.progress());
        let remaining = arithmetic(accounting.progress().checked_sub(on_schedule))?;
        let at_floor = arithmetic(amount.checked_sub(on_schedule))?;
        let next_state = MarketState::new(
            state.segmentation().contract_to(remaining)?,
            Accounting::new(
                remaining,
                arithmetic(accounting.circulating_supply().checked_sub(amount))?,
            ),
        )?;

        let mut reserve = constant_area(state.segmentation().floor().into(), at_floor)?;
        if on_schedule != NonNeg::zero() {
            reserve = add_bounds(
                reserve,
                self.compile(state)?.integrate(remaining, on_schedule)?,
            )?;
        }
        Ok(TradeQuote {
            next_state,
            reserve,
        })
    }

    fn quote_issue_at_floor(
        &self,
        state: &MarketState<N>,
        amount: Pos<N>,
    ) -> Result<TradeQuote<N>> {
        let accounting = state.accounting();
        let next_state = MarketState::new(
            state.segmentation(),
            Accounting::new(
                accounting.progress(),
                arithmetic(accounting.circulating_supply().checked_add(amount))?,
            ),
        )?;
        let reserve = constant_area(state.segmentation().floor().into(), amount.into())?;
        Ok(TradeQuote {
            next_state,
            reserve,
        })
    }

    fn quote_redeem_at_floor(
        &self,
        state: &MarketState<N>,
        amount: Pos<N>,
    ) -> Result<TradeQuote<N>> {
        let amount = NonNeg::from(amount);
        if amount > state.floor_tail()? {
            return Err(Error::InsufficientFloorTail);
        }
        let accounting = state.accounting();
        let next_state = MarketState::new(
            state.segmentation(),
            Accounting::new(
                accounting.progress(),
                arithmetic(accounting.circulating_supply().checked_sub(amount))?,
            ),
        )?;
        let reserve = constant_area(state.segmentation().floor().into(), amount)?;
        Ok(TradeQuote {
            next_state,
            reserve,
        })
    }

    fn verify_floor_raise(
        &self,
        state: &MarketState<N>,
        proposal: FloorRaiseProposal<N>,
        policy: FloorRaisePolicy<N>,
    ) -> Result<FloorRaiseQuote<N>> {
        let old = state.segmentation();
        if proposal.floor <= old.floor() {
            return Err(Error::FloorNotRaised);
        }
        let candidate = Segmentation::new(proposal.floor, proposal.ramp_end, proposal.ramp_width)?;
        if candidate.ramp_end() < old.ramp_end() || candidate.ramp_start()? < old.ramp_start()? {
            return Err(Error::RampMovedBackward);
        }
        if candidate.ramp_width() < old.ramp_width() {
            return Err(Error::RampNarrowed);
        }
        let next_state = MarketState::new(candidate, state.accounting())?;
        let previous = self.compile(state)?;
        let proposed = self.compile(&next_state)?;

        // Above the new ramp end both mains use the same unscaled characteristic.
        // Their difference at this point is their constant vertical displacement.
        let main_change = arithmetic(
            proposed
                .y(candidate.ramp_end())?
                .into_raw()
                .checked_sub(previous.y(candidate.ramp_end())?.into_raw()),
        )?;
        if main_change.lower() < arithmetic(policy.max_price_change.get().checked_neg())?
            || main_change.upper() > policy.max_price_change.get()
        {
            return Err(Error::MainCurveChanged);
        }

        let backing_change = arithmetic(
            backing(&proposed, &next_state)?
                .into_raw()
                .checked_sub(backing(&previous, state)?.into_raw()),
        )?;
        // The whole enclosure must certify nonincreasing backing. Merely
        // overlapping zero would also admit an unfunded increase.
        if backing_change.upper() > N::ZERO {
            return Err(Error::BackingIncreased);
        }
        if backing_change.lower() < arithmetic(policy.max_reserve_surplus.get().checked_neg())? {
            return Err(Error::ReserveSurplusExceeded);
        }
        Ok(FloorRaiseQuote {
            next_state,
            main_change,
            backing_change,
        })
    }
}

fn backing<N: SignedFixed, K: MonotoneCalculator<N>>(
    calculator: &ScheduleCalculator<N, K>,
    state: &MarketState<N>,
) -> Result<Bounds<NonNeg<N>>> {
    let floor_area = constant_area(state.segmentation().floor().into(), state.floor_tail()?)?;
    let start = state.segmentation().ramp_start()?;
    let width = arithmetic(state.accounting().progress().checked_sub(start))?;
    add_bounds(floor_area, calculator.integrate(start, width)?)
}

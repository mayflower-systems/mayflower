//! Persistable segmented schedule state, quoting, and state transitions.
//!
//! Geometry is an immutable value owned by [`crate::geometry`]. This module
//! translates total-supply coordinates, delegates geometric transformations,
//! and compiles disposable calculation views just in time.

use mayf_characteristic::{AffineAnchor, AffineTransform, Characteristic};
use mayf_fixed_number::{Bounds, FixedNumber, MathResult, NonNeg, Pos, SignedFixed};
use mayf_monotone::MonotoneCalculator;

use crate::calculator::{CompiledSegmentedSchedule, calculator_result};
use crate::geometry::{Segmentation, SegmentationError, SegmentedRaiseFloorProposal};

/// A failure while pricing or evolving a schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleError {
    InvalidSegmentation,
    InvalidInterval,
    CharacteristicOutOfDomain,
    NegativePrice,
    InsufficientSupply,
    /// A floor-raise proposal cannot certify a floor increase of at least
    /// one quantum.
    FloorRaiseTooSmall,
    /// A floor raise may not reshape the curve at or beyond the current
    /// schedule supply.
    FloorRaiseBeyondCurrentSupply,
    CalculatorInvariantViolation,
    ArithmeticFailure,
}

pub type ScheduleResult<T> = Result<T, ScheduleError>;

/// Principal returned by a schedule for an AVA issuance, before action-level fees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuyQuote<N> {
    pub ava_out: NonNeg<N>,
    pub reserve_in: NonNeg<N>,
}

/// Principal returned by a schedule for an AVA redemption, before action-level fees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SellQuote<N> {
    pub ava_in: NonNeg<N>,
    pub reserve_out: NonNeg<N>,
}

/// A candidate floor raise whose floor was derived from the continuity identity.
/// The caller accepts it after checking its required reserve against backing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloorRaise<N, S> {
    pub next_schedule: S,
    /// The derived floor: the old floor plus the largest increase the
    /// continuity identity certifies for the proposed boundaries.
    pub new_floor: NonNeg<N>,
    /// Certified upper bound on the live-price decrease. The construction
    /// guarantees the live price cannot increase.
    pub max_live_price_decrease: NonNeg<N>,
}

/// Prices and evolves a schedule at the current circulating AVA supply.
pub trait Schedule<N>: Sized
where
    N: FixedNumber,
{
    /// Maximum AVA that ordinary curve redemption may remove.
    fn max_sellable_ava(&self, current_supply: NonNeg<N>) -> ScheduleResult<NonNeg<N>>;

    /// Certified spot-price bounds at the current circulating supply.
    fn spot_price(&self, current_supply: NonNeg<N>) -> ScheduleResult<Bounds<NonNeg<N>>>;

    /// Certified reserve bounds required to back the circulating supply.
    fn required_reserve(&self, current_supply: NonNeg<N>) -> ScheduleResult<Bounds<NonNeg<N>>>;

    fn quote_buy_with_exact_ava_out(
        &self,
        current_supply: NonNeg<N>,
        ava_out: NonNeg<N>,
    ) -> ScheduleResult<BuyQuote<N>>;

    fn quote_sell_with_exact_ava_in(
        &self,
        current_supply: NonNeg<N>,
        ava_in: NonNeg<N>,
    ) -> ScheduleResult<SellQuote<N>>;

    /// Return the next schedule after an accepted issuance quote.
    fn apply_buy(self, current_supply: NonNeg<N>, quote: BuyQuote<N>) -> ScheduleResult<Self>;

    /// Return the next schedule after an accepted redemption quote.
    fn apply_sell(self, current_supply: NonNeg<N>, quote: SellQuote<N>) -> ScheduleResult<Self>;
}

/// A schedule with a guaranteed minimum redemption price.
pub trait FloorSchedule<N>: Sized
where
    N: FixedNumber,
{
    fn floor(&self) -> NonNeg<N>;

    fn apply_redeem_at_floor(self, amount_redeemed: NonNeg<N>) -> ScheduleResult<Self>;

    fn apply_issue_at_floor(self, amount_issued: NonNeg<N>) -> ScheduleResult<Self>;
}

/// A floor schedule that can construct a price-preserving floor raise.
pub trait RaisableFloorSchedule<N>: FloorSchedule<N>
where
    N: FixedNumber,
{
    type Proposal;

    fn raise_floor(
        self,
        current_supply: NonNeg<N>,
        proposal: Self::Proposal,
    ) -> ScheduleResult<FloorRaise<N, Self>>;
}

/// Persistable schedule state. Affine views are derived only by the private
/// just-in-time compiler.
///
/// `horizontal_shift` is outstanding supply issued at the floor rather than
/// bought on the curve. It translates the ramp and main to the right while
/// leaving an equally wide floor segment behind. Public supply coordinates are
/// total circulating supply; curve trades may not descend below this offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentedSchedule<N, C>
where
    N: SignedFixed,
    C: Characteristic<N>,
{
    characteristic: C,
    segmentation: Segmentation<N>,
    horizontal_shift: NonNeg<N>,
}

impl<N, C> SegmentedSchedule<N, C>
where
    N: SignedFixed,
    C: Characteristic<N>,
{
    /// Construct an operational schedule after validating its affine anchors.
    pub fn new(characteristic: C, segmentation: Segmentation<N>) -> ScheduleResult<Self> {
        Self::with_horizontal_shift(characteristic, segmentation, NonNeg::zero())
    }

    /// Reconstruct an operational schedule after validating persisted state.
    pub fn with_horizontal_shift(
        characteristic: C,
        segmentation: Segmentation<N>,
        horizontal_shift: NonNeg<N>,
    ) -> ScheduleResult<Self> {
        let schedule = Self::from_parts(characteristic, segmentation, horizontal_shift);
        schedule.validate()?;
        Ok(schedule)
    }

    pub const fn characteristic(&self) -> &C {
        &self.characteristic
    }

    pub const fn segmentation(&self) -> Segmentation<N> {
        self.segmentation
    }

    /// Outstanding supply issued at the floor. This is also the lowest supply
    /// coordinate reachable by ordinary curve sells.
    pub const fn horizontal_shift(&self) -> NonNeg<N> {
        self.horizontal_shift
    }

    fn with_segmentation(self, segmentation: Segmentation<N>) -> Self {
        Self::from_parts(self.characteristic, segmentation, self.horizontal_shift)
    }

    pub fn ramp_start(&self) -> ScheduleResult<NonNeg<N>> {
        self.shifted_coordinate(self.segmentation.ramp_start())
    }

    pub fn ramp_end(&self) -> ScheduleResult<NonNeg<N>> {
        self.shifted_coordinate(self.segmentation.ramp_end())
    }

    /// Ensure the disposable affine calculators and their shared boundary
    /// anchor can be evaluated. Increasingness is part of the
    /// [`Characteristic`] implementation contract and is not sampled here.
    pub fn validate(&self) -> ScheduleResult<()> {
        self.compile().map(|_| ())
    }

    pub fn y(&self, x: NonNeg<N>) -> ScheduleResult<Bounds<NonNeg<N>>> {
        self.compile()?.y(x)
    }

    /// Enclose reserve area over `[lo, lo + width]`.
    pub fn integrate(&self, lo: NonNeg<N>, width: NonNeg<N>) -> ScheduleResult<Bounds<NonNeg<N>>> {
        if width == NonNeg::zero() {
            return Ok(Bounds::exact(NonNeg::zero()));
        }
        self.compile()?.integrate(lo, width)
    }

    /// Raise the floor by the exact increase the continuity identity
    /// certifies for the proposed boundaries.
    ///
    /// The floor is derived, not supplied. With ramp scalar `k` and neutral
    /// characteristic `f`, keeping the reshaped curve vertically aligned with
    /// the old curve at and beyond the reshaped region requires
    ///
    /// ```text
    /// floor_increase = k*(f(x1') - f(x1)) - (k - 1)*(f(x2') - f(x2))
    /// ```
    ///
    /// The increase is taken as the lower edge of that identity's certified
    /// enclosure, so the live price provably cannot rise; it can fall by at
    /// most the enclosure width, which is returned as
    /// [`FloorRaise::max_live_price_decrease`].
    ///
    /// `current_supply` must sit at or beyond the reshaped region: the
    /// proposed ramp end when the ramp end advances, the proposed ramp start
    /// otherwise. Behind those boundaries the curve changes shape and no
    /// vertical alignment could preserve the live price. Reserve policy
    /// belongs to the calling action: an area-preserving raise must also
    /// preserve required reserve, while a surplus-funded raise may increase
    /// it only within available backing.
    pub fn raise_floor(
        self,
        current_supply: NonNeg<N>,
        proposal: SegmentedRaiseFloorProposal<N>,
    ) -> ScheduleResult<FloorRaise<N, Self>> {
        let (new_ramp_start, new_ramp_end) = self
            .segmentation
            .proposed_boundaries(proposal)
            .map_err(map_segmentation_error)?;

        let reshaped_through = if new_ramp_end == self.segmentation.ramp_end() {
            new_ramp_start
        } else {
            new_ramp_end
        };
        if current_supply < self.shifted_coordinate(reshaped_through)? {
            return Err(ScheduleError::FloorRaiseBeyondCurrentSupply);
        }

        let increase = self.certified_floor_increase(new_ramp_start, new_ramp_end)?;
        if increase.lower().get() < N::QUANTUM {
            return Err(ScheduleError::FloorRaiseTooSmall);
        }
        let max_live_price_decrease = increase
            .upper()
            .checked_sub(increase.lower())
            .map_err(|_| ScheduleError::CalculatorInvariantViolation)?;
        let new_floor = self
            .segmentation
            .floor()
            .checked_add(increase.lower())
            .map_err(|_| ScheduleError::ArithmeticFailure)?;

        let segmentation = self
            .segmentation
            .raise_floor(new_floor, proposal)
            .map_err(map_segmentation_error)?;
        let candidate = self.with_segmentation(segmentation);
        candidate.validate()?;

        Ok(FloorRaise {
            next_schedule: candidate,
            new_floor,
            max_live_price_decrease,
        })
    }

    /// Certified enclosure of the continuity identity's right-hand side,
    /// `k*(f(x1') - f(x1)) - (k - 1)*(f(x2') - f(x2))`, over the neutral
    /// characteristic.
    fn certified_floor_increase(
        &self,
        new_ramp_start: NonNeg<N>,
        new_ramp_end: NonNeg<N>,
    ) -> ScheduleResult<Bounds<NonNeg<N>>> {
        let scalar = NonNeg::from(self.segmentation.ramp_scalar());

        let start_gain = characteristic_gap(
            &self.characteristic,
            self.segmentation.ramp_start(),
            new_ramp_start,
        )?;
        let raise_budget = scale_bounds(start_gain, scalar)?;
        if new_ramp_end == self.segmentation.ramp_end() {
            return Ok(raise_budget);
        }

        // `AboveOne` makes `k - 1` positive.
        let excess_scalar = arithmetic(scalar.checked_sub(NonNeg::one()))?;
        let end_gain = characteristic_gap(
            &self.characteristic,
            self.segmentation.ramp_end(),
            new_ramp_end,
        )?;
        let substituted = scale_bounds(end_gain, excess_scalar)?;

        // Outward difference. A lower edge that cannot stay non-negative
        // cannot certify a floor increase at all.
        let lower = raise_budget
            .lower()
            .checked_sub(substituted.upper())
            .map_err(|_| ScheduleError::FloorRaiseTooSmall)?;
        let upper = raise_budget
            .upper()
            .checked_sub(substituted.lower())
            .map_err(|_| ScheduleError::CalculatorInvariantViolation)?;

        Bounds::new(lower, upper).map_err(|_| ScheduleError::CalculatorInvariantViolation)
    }

    pub fn quote_buy_with_exact_ava_out(
        &self,
        current_supply: NonNeg<N>,
        ava_out: NonNeg<N>,
    ) -> ScheduleResult<BuyQuote<N>> {
        if ava_out == NonNeg::zero() {
            return Ok(BuyQuote {
                ava_out,
                reserve_in: NonNeg::zero(),
            });
        }
        self.require_trading_supply(current_supply)?;
        let calculator = self.compile()?;
        let reserve_in = calculator.integrate(current_supply, ava_out)?.upper();

        Ok(BuyQuote {
            ava_out,
            reserve_in,
        })
    }

    pub fn quote_sell_with_exact_ava_in(
        &self,
        current_supply: NonNeg<N>,
        ava_in: NonNeg<N>,
    ) -> ScheduleResult<SellQuote<N>> {
        if ava_in == NonNeg::zero() {
            return Ok(SellQuote {
                ava_in,
                reserve_out: NonNeg::zero(),
            });
        }
        let curve_issued_supply = self.max_sellable_ava(current_supply)?;
        if ava_in > curve_issued_supply {
            return Err(ScheduleError::InsufficientSupply);
        }

        let calculator = self.compile()?;
        let next_supply = arithmetic(current_supply.checked_sub(ava_in))?;
        let reserve_out = calculator.integrate(next_supply, ava_in)?.lower();

        Ok(SellQuote {
            ava_in,
            reserve_out,
        })
    }

    /// AVA bought on the curve and therefore removable by an ordinary redemption.
    pub fn max_sellable_ava(&self, current_supply: NonNeg<N>) -> ScheduleResult<NonNeg<N>> {
        self.require_trading_supply(current_supply)?;
        current_supply
            .checked_sub(self.horizontal_shift)
            .map_err(|_| ScheduleError::InsufficientSupply)
    }

    pub fn contract_to(self, supply: NonNeg<N>) -> ScheduleResult<Self> {
        self.require_trading_supply(supply)?;

        let curve_supply = supply
            .checked_sub(self.horizontal_shift)
            .map_err(|_| ScheduleError::InsufficientSupply)?;
        let segmentation = self.segmentation.contract_to_curve_supply(curve_supply);
        if segmentation == self.segmentation {
            return Ok(self);
        }

        let contracted = Self::from_parts(self.characteristic, segmentation, self.horizontal_shift);
        contracted.validate()?;

        Ok(contracted)
    }

    pub fn shift_right(self, amount: NonNeg<N>) -> ScheduleResult<Self> {
        if amount == NonNeg::zero() {
            return Ok(self);
        }
        let horizontal_shift = self
            .horizontal_shift
            .checked_add(amount)
            .map_err(|_| ScheduleError::ArithmeticFailure)?;
        let shifted = Self::from_parts(self.characteristic, self.segmentation, horizontal_shift);
        shifted.validate()?;
        Ok(shifted)
    }

    pub fn shift_left(self, amount: NonNeg<N>) -> ScheduleResult<Self> {
        if amount == NonNeg::zero() {
            return Ok(self);
        }
        let horizontal_shift = self
            .horizontal_shift
            .checked_sub(amount)
            .map_err(|_| ScheduleError::InsufficientSupply)?;
        let shifted = Self::from_parts(self.characteristic, self.segmentation, horizontal_shift);
        shifted.validate()?;
        Ok(shifted)
    }

    const fn from_parts(
        characteristic: C,
        segmentation: Segmentation<N>,
        horizontal_shift: NonNeg<N>,
    ) -> Self {
        Self {
            characteristic,
            segmentation,
            horizontal_shift,
        }
    }

    fn compile(&self) -> ScheduleResult<CompiledSegmentedSchedule<N, C::Calculator>> {
        CompiledSegmentedSchedule::compile(
            &self.characteristic,
            self.segmentation,
            self.horizontal_shift,
        )
    }

    fn shifted_coordinate(&self, coordinate: NonNeg<N>) -> ScheduleResult<NonNeg<N>> {
        coordinate
            .checked_add(self.horizontal_shift)
            .map_err(|_| ScheduleError::ArithmeticFailure)
    }

    fn require_trading_supply(&self, supply: NonNeg<N>) -> ScheduleResult<()> {
        if supply < self.horizontal_shift {
            Err(ScheduleError::InsufficientSupply)
        } else {
            Ok(())
        }
    }
}

impl<N, C> Schedule<N> for SegmentedSchedule<N, C>
where
    N: SignedFixed,
    C: Characteristic<N>,
{
    fn max_sellable_ava(&self, current_supply: NonNeg<N>) -> ScheduleResult<NonNeg<N>> {
        SegmentedSchedule::max_sellable_ava(self, current_supply)
    }

    fn spot_price(&self, current_supply: NonNeg<N>) -> ScheduleResult<Bounds<NonNeg<N>>> {
        SegmentedSchedule::max_sellable_ava(self, current_supply)?;
        self.y(current_supply)
    }

    fn required_reserve(&self, current_supply: NonNeg<N>) -> ScheduleResult<Bounds<NonNeg<N>>> {
        SegmentedSchedule::max_sellable_ava(self, current_supply)?;
        self.integrate(NonNeg::zero(), current_supply)
    }

    fn quote_buy_with_exact_ava_out(
        &self,
        current_supply: NonNeg<N>,
        ava_out: NonNeg<N>,
    ) -> ScheduleResult<BuyQuote<N>> {
        SegmentedSchedule::quote_buy_with_exact_ava_out(self, current_supply, ava_out)
    }

    fn quote_sell_with_exact_ava_in(
        &self,
        current_supply: NonNeg<N>,
        ava_in: NonNeg<N>,
    ) -> ScheduleResult<SellQuote<N>> {
        SegmentedSchedule::quote_sell_with_exact_ava_in(self, current_supply, ava_in)
    }

    fn apply_buy(self, _current_supply: NonNeg<N>, _quote: BuyQuote<N>) -> ScheduleResult<Self> {
        Ok(self)
    }

    fn apply_sell(self, current_supply: NonNeg<N>, quote: SellQuote<N>) -> ScheduleResult<Self> {
        if quote.ava_in == NonNeg::zero() {
            return Ok(self);
        }
        let next_supply = current_supply
            .checked_sub(quote.ava_in)
            .map_err(|_| ScheduleError::InsufficientSupply)?;

        self.contract_to(next_supply)
    }
}

impl<N, C> FloorSchedule<N> for SegmentedSchedule<N, C>
where
    N: SignedFixed,
    C: Characteristic<N>,
{
    fn floor(&self) -> NonNeg<N> {
        self.segmentation().floor()
    }

    fn apply_redeem_at_floor(self, amount_redeemed: NonNeg<N>) -> ScheduleResult<Self> {
        self.shift_left(amount_redeemed)
    }

    fn apply_issue_at_floor(self, amount_issued: NonNeg<N>) -> ScheduleResult<Self> {
        self.shift_right(amount_issued)
    }
}

impl<N, C> RaisableFloorSchedule<N> for SegmentedSchedule<N, C>
where
    N: SignedFixed,
    C: Characteristic<N>,
{
    type Proposal = SegmentedRaiseFloorProposal<N>;

    fn raise_floor(
        self,
        current_supply: NonNeg<N>,
        proposal: Self::Proposal,
    ) -> ScheduleResult<FloorRaise<N, Self>> {
        SegmentedSchedule::raise_floor(self, current_supply, proposal)
    }
}

fn map_segmentation_error(error: SegmentationError) -> ScheduleError {
    match error {
        SegmentationError::FloorDecrease | SegmentationError::RampStartsAfterEnd => {
            ScheduleError::InvalidSegmentation
        }
        SegmentationError::RampWidthExceedsEnd => ScheduleError::InvalidInterval,
        SegmentationError::ArithmeticFailure => ScheduleError::ArithmeticFailure,
    }
}

/// Enclose `f(hi) - f(lo)` using the characteristic's anchored difference
/// formula. Subtracting independently rounded neutral prices would lose
/// precision for a small advance far from the origin.
fn characteristic_gap<N, C>(
    characteristic: &C,
    lo: NonNeg<N>,
    hi: NonNeg<N>,
) -> ScheduleResult<Bounds<NonNeg<N>>>
where
    N: SignedFixed,
    C: Characteristic<N>,
{
    let affine = AffineTransform::new(
        Pos::one(),
        N::ZERO,
        NonNeg::one(),
        AffineAnchor::exact(lo, NonNeg::zero()),
    );
    calculator_result(characteristic.calculator(affine).y(hi))
}

/// Outward-rounded product of a certified enclosure and a non-negative
/// scalar.
fn scale_bounds<N>(
    bounds: Bounds<NonNeg<N>>,
    scalar: NonNeg<N>,
) -> ScheduleResult<Bounds<NonNeg<N>>>
where
    N: FixedNumber,
{
    arithmetic((bounds.bounded() * scalar).bounds())
}

fn arithmetic<T>(result: MathResult<T>) -> ScheduleResult<T> {
    result.map_err(|_| ScheduleError::ArithmeticFailure)
}

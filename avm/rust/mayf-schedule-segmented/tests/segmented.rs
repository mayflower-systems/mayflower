use mayf_characteristic::{AffineTransform, Characteristic};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{AboveOne, Bounds, DirectedRounding, FixedNumber, NonNeg, Pos};
use mayf_monotone::{MonotoneCalculator, MonotoneCalculatorError, MonotoneCalculatorResult};
use mayf_schedule_segmented::{
    FloorSchedule, RaisableFloorSchedule, Schedule, ScheduleError, Segmentation,
    SegmentedRaiseFloorProposal, SegmentedSchedule,
};

type Fx = I64F64;

fn n(value: i64) -> Fx {
    Fx::checked_from_i128(i128::from(value)).unwrap()
}

fn d(mantissa: i64, scale: u32) -> Fx {
    let denominator = core::num::NonZeroU128::new(10_u128.pow(scale)).unwrap();
    Fx::checked_from_ratio_round(i128::from(mantissa), denominator, DirectedRounding::Floor)
        .unwrap()
}

fn nn(value: i64) -> NonNeg<Fx> {
    NonNeg::new(n(value)).unwrap()
}

fn nnd(mantissa: i64, scale: u32) -> NonNeg<Fx> {
    NonNeg::new(d(mantissa, scale)).unwrap()
}

fn floor_raise_proposal(
    ramp_start_advance_by: NonNeg<Fx>,
    ramp_end_extension_by: NonNeg<Fx>,
) -> SegmentedRaiseFloorProposal<Fx> {
    SegmentedRaiseFloorProposal::new(
        Pos::try_from(ramp_start_advance_by).unwrap(),
        ramp_end_extension_by,
    )
}

fn linear_schedule() -> SegmentedSchedule<Fx, LinearCharacteristic<Fx>> {
    let segmentation =
        Segmentation::new(nn(5), nn(1), nn(3), AboveOne::new(d(15, 1)).unwrap()).unwrap();

    SegmentedSchedule::new(
        LinearCharacteristic::new(Pos::from_i32(2).unwrap()),
        segmentation,
    )
    .unwrap()
}

#[test]
fn affine_calculators_form_one_continuous_piecewise_schedule() {
    let schedule = linear_schedule();
    schedule.validate().unwrap();
    let segmentation = schedule.segmentation();

    assert_eq!(schedule.y(nn(0)).unwrap().lower(), segmentation.floor());
    assert_eq!(
        schedule.y(segmentation.ramp_start()).unwrap().lower(),
        segmentation.floor()
    );
    assert!(
        schedule.y(segmentation.ramp_end()).unwrap().lower()
            > schedule.y(segmentation.ramp_start()).unwrap().upper()
    );

    let supply = segmentation.ramp_end().checked_add(nn(1)).unwrap();
    assert_eq!(
        Schedule::required_reserve(&schedule, supply).unwrap(),
        schedule.integrate(nn(0), supply).unwrap()
    );
}

#[test]
fn reserve_quotes_cross_each_boundary_without_losing_safe_side_guarantees() {
    let schedule = linear_schedule();
    let supply = schedule
        .segmentation()
        .ramp_end()
        .checked_add(nn(1))
        .unwrap();
    let full_area = schedule.integrate(nn(0), supply).unwrap();

    let ava_buy = schedule
        .quote_buy_with_exact_ava_out(nn(0), supply)
        .unwrap();
    assert_eq!(ava_buy.reserve_in, full_area.upper());

    let ava_sell = schedule
        .quote_sell_with_exact_ava_in(supply, supply)
        .unwrap();
    assert_eq!(ava_sell.reserve_out, full_area.lower());

    assert_eq!(
        schedule.quote_sell_with_exact_ava_in(supply, supply.checked_add(nnd(1, 12)).unwrap(),),
        Err(ScheduleError::InsufficientSupply)
    );
}

#[test]
fn zero_quantities_are_identity_operations() {
    let schedule = linear_schedule();
    let zero = nn(0);

    assert_eq!(
        schedule.integrate(nn(2), zero).unwrap(),
        mayf_fixed_number::Bounds::exact(zero)
    );
    assert_eq!(
        schedule.quote_buy_with_exact_ava_out(nn(2), zero).unwrap(),
        mayf_schedule_segmented::BuyQuote {
            ava_out: zero,
            reserve_in: zero,
        }
    );
    assert_eq!(
        schedule.quote_sell_with_exact_ava_in(nn(2), zero).unwrap(),
        mayf_schedule_segmented::SellQuote {
            ava_in: zero,
            reserve_out: zero,
        }
    );
    assert_eq!(schedule.shift_right(zero).unwrap(), schedule);
    assert_eq!(schedule.shift_left(zero).unwrap(), schedule);
}

#[test]
fn schedule_contraction_reanchors_the_main_without_changing_the_live_price() {
    let schedule = linear_schedule();

    let inside_ramp = schedule.contract_to(nn(2)).unwrap();
    assert_eq!(inside_ramp.y(nn(2)).unwrap(), schedule.y(nn(2)).unwrap());
    assert!(inside_ramp.y(nn(3)).unwrap().upper() < schedule.y(nn(3)).unwrap().lower());

    let below_ramp_supply = nnd(5, 1);
    let below_ramp = schedule.contract_to(below_ramp_supply).unwrap();
    assert_eq!(
        below_ramp.y(below_ramp_supply).unwrap().lower(),
        schedule.segmentation().floor()
    );
    assert!(below_ramp.y(nn(1)).unwrap().lower() > schedule.segmentation().floor());

    let above_ramp = schedule
        .segmentation()
        .ramp_end()
        .checked_add(nn(1))
        .unwrap();
    assert_eq!(schedule.contract_to(above_ramp).unwrap(), schedule);
}

#[test]
fn schedule_contract_constructs_a_floor_raise_with_a_derived_floor() {
    let schedule = linear_schedule();
    let old = schedule.segmentation();
    // Continuity across the reshape certifies exactly one unit of increase:
    // 1.5*(f(2) - f(1)) - 0.5*(f(5) - f(3)) = 3 - 2 = 1.
    let expected_floor = old.floor().checked_add(nn(1)).unwrap();
    let ramp_end_increase = nn(2);
    let new_ramp_end = old.ramp_end().checked_add(ramp_end_increase).unwrap();
    let new_ramp_width = nn(3);
    let raised = RaisableFloorSchedule::raise_floor(
        schedule,
        nn(6),
        floor_raise_proposal(nn(1), ramp_end_increase),
    )
    .unwrap();

    assert_eq!(raised.new_floor, expected_floor);
    assert_eq!(
        raised.max_live_price_decrease,
        nn(0),
        "linear evaluations are exact, so the certificate is exact"
    );
    let raised_schedule = raised.next_schedule;
    assert_eq!(raised_schedule.segmentation().floor(), expected_floor);
    assert_eq!(raised_schedule.segmentation().ramp_end(), new_ramp_end);
    assert_eq!(
        raised_schedule.segmentation().ramp_width().unwrap(),
        new_ramp_width
    );
    let main_sample = new_ramp_end.checked_add(nn(1)).unwrap();
    assert_eq!(
        raised_schedule.y(main_sample).unwrap(),
        schedule.y(main_sample).unwrap()
    );
}

#[test]
fn floor_raise_may_collapse_the_ramp_at_its_existing_end() {
    let schedule = linear_schedule();
    let ramp_end = schedule.segmentation().ramp_end();
    // Collapsing the ramp derives a floor equal to the old curve's value at
    // the unchanged ramp end.
    let expected_floor = schedule.y(ramp_end).unwrap().lower();
    let raised =
        RaisableFloorSchedule::raise_floor(schedule, nn(4), floor_raise_proposal(nn(2), nn(0)))
            .unwrap();

    assert_eq!(raised.new_floor, expected_floor);
    let segmentation = raised.next_schedule.segmentation();
    assert_eq!(segmentation.floor(), expected_floor);
    assert_eq!(segmentation.ramp_start(), ramp_end);
    assert_eq!(segmentation.ramp_end(), ramp_end);
    assert_eq!(segmentation.ramp_width().unwrap(), nn(0));
    let main_sample = ramp_end.checked_add(nn(1)).unwrap();
    assert_eq!(
        raised.next_schedule.y(main_sample).unwrap(),
        schedule.y(main_sample).unwrap()
    );
}

#[test]
fn unchanged_ramp_end_allows_a_raise_below_the_old_ramp_end() {
    let schedule = linear_schedule();
    let old = schedule.segmentation();
    let new_ramp_start = old.ramp_start().checked_add(nn(1)).unwrap();
    let expected_floor = schedule.y(new_ramp_start).unwrap().lower();
    let old_area = Schedule::required_reserve(&schedule, new_ramp_start).unwrap();
    let raised = RaisableFloorSchedule::raise_floor(
        schedule,
        new_ramp_start,
        floor_raise_proposal(nn(1), nn(0)),
    )
    .unwrap();

    assert_eq!(raised.new_floor, expected_floor);
    let segmentation = raised.next_schedule.segmentation();
    assert_eq!(segmentation.floor(), expected_floor);
    assert_eq!(segmentation.ramp_start(), new_ramp_start);
    assert_eq!(segmentation.ramp_end(), old.ramp_end());
    assert!(
        Schedule::required_reserve(&raised.next_schedule, new_ramp_start)
            .unwrap()
            .lower()
            > old_area.upper()
    );
}

#[test]
fn floor_raise_rejects_supply_behind_the_reshaped_region() {
    // An advancing ramp end reshapes the curve through the proposed end.
    let result = RaisableFloorSchedule::raise_floor(
        linear_schedule(),
        nn(4),
        floor_raise_proposal(nn(1), nn(2)),
    );
    assert_eq!(
        result.unwrap_err(),
        ScheduleError::FloorRaiseBeyondCurrentSupply
    );

    // A fixed ramp end still reshapes the curve through the ramp start.
    let result = RaisableFloorSchedule::raise_floor(
        linear_schedule(),
        nn(1),
        floor_raise_proposal(nn(1), nn(0)),
    );
    assert_eq!(
        result.unwrap_err(),
        ScheduleError::FloorRaiseBeyondCurrentSupply
    );
}

#[test]
fn floor_raise_rejects_a_sub_quantum_increase() {
    // The ramp start barely advances while the advancing end substitutes a
    // full strip of main for ramp: the identity's certified lower edge goes
    // negative, so no floor increase can be certified.
    let quantum = NonNeg::new(Fx::QUANTUM).unwrap();
    let result = RaisableFloorSchedule::raise_floor(
        linear_schedule(),
        nn(6),
        floor_raise_proposal(quantum, nn(2)),
    );

    assert_eq!(result.unwrap_err(), ScheduleError::FloorRaiseTooSmall);
}

#[test]
fn floor_issuance_and_redemption_translate_the_schedule_affinely() {
    let schedule = linear_schedule();
    let amount = nn(2);
    let issued = FloorSchedule::apply_issue_at_floor(schedule, amount).unwrap();

    assert_eq!(issued.horizontal_shift(), amount);
    assert_eq!(
        issued.ramp_start().unwrap(),
        schedule
            .segmentation()
            .ramp_start()
            .checked_add(amount)
            .unwrap()
    );
    assert_eq!(
        issued.ramp_end().unwrap(),
        schedule
            .segmentation()
            .ramp_end()
            .checked_add(amount)
            .unwrap()
    );

    let curve_supply = schedule
        .segmentation()
        .ramp_end()
        .checked_add(nn(1))
        .unwrap();
    let total_supply = curve_supply.checked_add(amount).unwrap();
    assert_eq!(
        issued.y(total_supply).unwrap(),
        schedule.y(curve_supply).unwrap()
    );
    assert_eq!(
        issued.integrate(amount, curve_supply).unwrap(),
        schedule.integrate(nn(0), curve_supply).unwrap()
    );

    let old_reserve = schedule.integrate(nn(0), curve_supply).unwrap();
    let new_reserve = issued.integrate(nn(0), total_supply).unwrap();
    let floor_backing = schedule
        .segmentation()
        .floor()
        .checked_mul_floor(amount)
        .unwrap();
    assert_eq!(
        new_reserve.lower(),
        old_reserve.lower().checked_add(floor_backing).unwrap()
    );
    assert_eq!(
        new_reserve.upper(),
        old_reserve.upper().checked_add(floor_backing).unwrap()
    );

    assert_eq!(
        issued.quote_sell_with_exact_ava_in(
            total_supply,
            curve_supply.checked_add(nnd(1, 12)).unwrap(),
        ),
        Err(ScheduleError::InsufficientSupply)
    );
    let unshifted_sell = schedule
        .quote_sell_with_exact_ava_in(curve_supply, curve_supply)
        .unwrap();
    let curve_sell = issued
        .quote_sell_with_exact_ava_in(total_supply, curve_supply)
        .unwrap();
    assert_eq!(curve_sell.reserve_out, unshifted_sell.reserve_out);

    assert_eq!(
        FloorSchedule::apply_redeem_at_floor(issued, amount).unwrap(),
        schedule
    );
    assert_eq!(
        FloorSchedule::apply_redeem_at_floor(issued, amount.checked_add(nnd(1, 12)).unwrap(),),
        Err(ScheduleError::InsufficientSupply)
    );
}

// --- Certificate behavior under inexact enclosures -----------------------

/// A linear characteristic whose point evaluations are artificially widened
/// by one quantum on each side to test a characteristic with looser enclosures.
struct PaddedLinear(LinearCharacteristic<Fx>);

struct PaddedCalculator(<LinearCharacteristic<Fx> as Characteristic<Fx>>::Calculator);

impl Characteristic<Fx> for PaddedLinear {
    type Calculator = PaddedCalculator;

    fn calculator(&self, affine: AffineTransform<Fx>) -> Self::Calculator {
        PaddedCalculator(self.0.calculator(affine))
    }
}

impl MonotoneCalculator<Fx> for PaddedCalculator {
    fn y(&self, x: NonNeg<Fx>) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        let exact = self.0.y(x)?;
        let quantum = NonNeg::new(Fx::QUANTUM).unwrap();
        let lower = exact.lower().checked_sub(quantum).unwrap_or(NonNeg::zero());
        let upper = exact
            .upper()
            .checked_add(quantum)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        Ok(Bounds::new(lower, upper).unwrap())
    }

    fn integrate(
        &self,
        lo: NonNeg<Fx>,
        width: NonNeg<Fx>,
    ) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        self.0.integrate(lo, width)
    }
}

#[test]
fn floor_raise_certificate_measures_enclosure_width_one_sided() {
    let segmentation =
        Segmentation::new(nn(5), nn(1), nn(3), AboveOne::new(d(15, 1)).unwrap()).unwrap();
    let padded = SegmentedSchedule::new(
        PaddedLinear(LinearCharacteristic::new(Pos::from_i32(2).unwrap())),
        segmentation,
    )
    .unwrap();
    let exact_reference = linear_schedule();

    let proposal = floor_raise_proposal(nn(1), nn(0));
    let padded_raise = padded.raise_floor(nn(2), proposal).unwrap();
    let exact_raise = exact_reference.raise_floor(nn(2), proposal).unwrap();

    // One anchored difference is widened by one quantum per side. Multiplying
    // by 1.5 rounds each edge out by two quanta, making the certificate four.
    let four_quanta = NonNeg::new(
        Fx::QUANTUM
            .checked_mul_ceil(Fx::checked_from_i128(4).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(padded_raise.max_live_price_decrease, four_quanta);
    assert_eq!(exact_raise.max_live_price_decrease, nn(0));

    // The derived floor takes the identity's certified lower edge, so it can
    // only sit below the exact continuity floor — the live price can only
    // move down — and by no more than the certificate itself.
    assert!(padded_raise.new_floor <= exact_raise.new_floor);
    let shortfall = exact_raise
        .new_floor
        .checked_sub(padded_raise.new_floor)
        .unwrap();
    assert!(shortfall <= padded_raise.max_live_price_decrease);
}

#[test]
fn a_quantum_floor_advance_uses_an_anchored_characteristic_difference() {
    let one_raw = 1i128 << 64;
    let raw = |value| NonNeg::new(Fx::from_scaled_i128(value)).unwrap();
    let start = raw(1_000_000 * one_raw + 1);
    let schedule = SegmentedSchedule::new(
        LinearCharacteristic::try_new(raw(one_raw + 1).get()).unwrap(),
        Segmentation::new(
            nn(1),
            start,
            start.checked_add(nn(1)).unwrap(),
            AboveOne::new(n(2)).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let raised = schedule
        .raise_floor(
            schedule.ramp_end().unwrap(),
            SegmentedRaiseFloorProposal::new(Pos::new(Fx::QUANTUM).unwrap(), nn(0)),
        )
        .unwrap();
    assert_eq!(raised.new_floor, raw(one_raw + 2));
    assert_eq!(raised.max_live_price_decrease, raw(2));
}

#[test]
fn area_preserving_and_surplus_funded_raises_share_the_same_geometry() {
    let schedule = SegmentedSchedule::new(
        LinearCharacteristic::new(Pos::from_i32(8).unwrap()),
        Segmentation::new(nn(1), nn(0), nn(0), AboveOne::new(d(15625, 4)).unwrap()).unwrap(),
    )
    .unwrap();
    let supply = nn(6);
    let original_reserve = schedule.required_reserve(supply).unwrap();
    assert_eq!(original_reserve, Bounds::exact(nn(150)));
    // This proposal redistributes area exactly: floor rises from 1 to 16,
    // with new boundaries 3 and 5, and the main price stays 49 at supply 6.
    let neutral = schedule
        .raise_floor(supply, floor_raise_proposal(nn(3), nn(2)))
        .unwrap();
    assert_eq!(neutral.new_floor, nn(16));
    assert_eq!(neutral.max_live_price_decrease, nn(0));
    assert_eq!(
        neutral.next_schedule.required_reserve(supply).unwrap(),
        original_reserve
    );
    assert_eq!(
        neutral.next_schedule.spot_price(supply).unwrap(),
        Bounds::exact(nn(49))
    );
    // A collapsed ramp instead needs four units of additional backing. The
    // schedule constructs the candidate; the action must fund and accept it.
    let funded = schedule
        .raise_floor(supply, floor_raise_proposal(nn(1), nn(0)))
        .unwrap();
    assert_eq!(
        funded.next_schedule.required_reserve(supply).unwrap(),
        Bounds::exact(nn(154))
    );
    assert_eq!(
        funded.next_schedule.spot_price(supply).unwrap(),
        schedule.spot_price(supply).unwrap()
    );
}

#[test]
fn selling_into_the_ramp_then_buying_back_keeps_the_contracted_main() {
    let schedule = linear_schedule().shift_right(nn(2)).unwrap();
    let sell = schedule.quote_sell_with_exact_ava_in(nn(6), nn(2)).unwrap();
    assert_eq!(sell.reserve_out, nnd(215, 1));
    let contracted = Schedule::apply_sell(schedule, nn(6), sell).unwrap();
    assert_eq!(
        contracted.spot_price(nn(4)).unwrap(),
        schedule.spot_price(nn(4)).unwrap()
    );
    let buy = contracted
        .quote_buy_with_exact_ava_out(nn(4), nn(2))
        .unwrap();
    assert_eq!(buy.reserve_in, nn(20));
    let rebought = Schedule::apply_buy(contracted, nn(4), buy).unwrap();
    assert_eq!(rebought, contracted);
    assert_eq!(rebought.spot_price(nn(6)).unwrap(), Bounds::exact(nn(12)));
    assert_eq!(schedule.spot_price(nn(6)).unwrap(), Bounds::exact(nn(13)));

    let sell_all = rebought.quote_sell_with_exact_ava_in(nn(6), nn(4)).unwrap();
    let reset = Schedule::apply_sell(rebought, nn(6), sell_all).unwrap();
    assert_eq!(reset.ramp_start().unwrap(), nn(2));
    assert_eq!(reset.ramp_end().unwrap(), nn(2));
    assert_eq!(reset.spot_price(nn(2)).unwrap(), Bounds::exact(nn(5)));
    assert_eq!(
        reset.quote_sell_with_exact_ava_in(nn(2), nn(1)),
        Err(ScheduleError::InsufficientSupply)
    );
}

// Simulate a characteristic whose unit-scale view is outside its supported
// range, while the amplified ramp view remains usable. Construction must check
// both views and translate every calculator error at the schedule boundary.
struct MainFailure(MonotoneCalculatorError);
struct FailingCalculator {
    inner: <LinearCharacteristic<Fx> as Characteristic<Fx>>::Calculator,
    error: Option<MonotoneCalculatorError>,
}
impl Characteristic<Fx> for MainFailure {
    type Calculator = FailingCalculator;
    fn calculator(&self, affine: AffineTransform<Fx>) -> Self::Calculator {
        FailingCalculator {
            inner: LinearCharacteristic::new(Pos::one()).calculator(affine),
            error: (affine.y_scale() == NonNeg::one()).then_some(self.0),
        }
    }
}
impl MonotoneCalculator<Fx> for FailingCalculator {
    fn y(&self, x: NonNeg<Fx>) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        if let Some(error) = self.error {
            return Err(error);
        }
        self.inner.y(x)
    }
    fn integrate(
        &self,
        lo: NonNeg<Fx>,
        width: NonNeg<Fx>,
    ) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        if let Some(error) = self.error {
            return Err(error);
        }
        self.inner.integrate(lo, width)
    }
}

#[test]
fn construction_checks_the_main_anchor_and_maps_calculator_failures() {
    use MonotoneCalculatorError as Calc;
    let geometry = linear_schedule().segmentation();
    for (failure, expected) in [
        (
            Calc::InputOutOfDomain,
            ScheduleError::CharacteristicOutOfDomain,
        ),
        (Calc::InvalidInterval, ScheduleError::InvalidInterval),
        (Calc::NegativeOutput, ScheduleError::NegativePrice),
        (Calc::ArithmeticFailure, ScheduleError::ArithmeticFailure),
    ] {
        assert_eq!(
            SegmentedSchedule::new(MainFailure(failure), geometry).err(),
            Some(expected)
        );
    }
    let max_shift = NonNeg::new(Fx::MAX).unwrap();
    assert_eq!(
        linear_schedule().shift_right(max_shift),
        Err(ScheduleError::ArithmeticFailure)
    );
    assert_eq!(
        linear_schedule().integrate(NonNeg::new(Fx::MAX).unwrap(), nn(1)),
        Err(ScheduleError::ArithmeticFailure)
    );
}

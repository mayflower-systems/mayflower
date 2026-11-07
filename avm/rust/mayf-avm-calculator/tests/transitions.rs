use std::cell::RefCell;

use mayf_avm_calculator::{
    Accounting, AvmCalculator, CharacteristicEngine, CharacteristicEngineError,
    CharacteristicEngineResult, Error, FloorRaisePolicy, FloorRaiseProposal, LocalCharacteristic,
    MarketState, ScheduleCalculator, Segmentation, SpanRequest, SpanResponse,
};
use mayf_characteristic::CharacteristicError;
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::{I64F64, I64Scaled};
use mayf_fixed_number::{
    AboveOne, Bounds, DirectedRounding, FixedNumber, NonNeg, Pos, SignedFixed,
};

type N = I64F64;

fn nn(n: i128) -> NonNeg<N> {
    NonNeg::new(N::checked_from_i128(n).unwrap()).unwrap()
}

fn pos(n: i32) -> Pos<N> {
    Pos::from_i32(n).unwrap()
}

fn fraction(n: i128, d: u128) -> NonNeg<N> {
    NonNeg::new(
        N::checked_from_ratio_round(n, d.try_into().unwrap(), DirectedRounding::Floor).unwrap(),
    )
    .unwrap()
}

fn scalar() -> AboveOne<N> {
    AboveOne::new(fraction(25, 16).get()).unwrap()
}

fn engine() -> LocalCharacteristic<LinearCharacteristic<N>> {
    LocalCharacteristic::new(LinearCharacteristic::new(pos(8)))
}

fn calculator() -> AvmCalculator<N> {
    AvmCalculator::new(scalar())
}

fn negative(value: NonNeg<N>) -> N {
    value.get().checked_neg().unwrap()
}

fn proposal() -> FloorRaiseProposal<N> {
    FloorRaiseProposal {
        floor_increase_by: pos(15),
        ramp_end: nn(5),
        ramp_width: nn(2),
    }
}

fn raised() -> MarketState<N> {
    MarketState::new(
        Segmentation::new(nn(16), nn(5), nn(2), nn(1).get()).unwrap(),
        Accounting::new(nn(6), nn(6)),
    )
    .unwrap()
}

#[test]
fn ordinary_sales_retain_the_prefix_and_resume_buying_on_main() {
    let engine = engine();
    let calculator = calculator();
    for (remaining, width, rate, backing) in [
        (5, 2, nn(41), nn(105)),
        (4, 1, fraction(57, 2), fraction(281, 4)),
        (3, 0, nn(16), nn(48)),
        (2, 0, nn(16), nn(32)),
        (0, 0, nn(16), nn(0)),
    ] {
        let sale = calculator
            .quote_redeem(&raised(), pos(6 - remaining), &engine)
            .unwrap();
        assert_eq!(
            sale.next_state.accounting(),
            Accounting::new(nn(remaining.into()), nn(remaining.into()))
        );
        assert_eq!(
            sale.next_state.segmentation().ramp_end(),
            nn(remaining.into())
        );
        assert_eq!(sale.next_state.segmentation().ramp_width(), nn(width));
        assert_eq!(
            calculator.price(&sale.next_state, &engine).unwrap(),
            Bounds::exact(rate)
        );
        assert_eq!(
            calculator.backing(&sale.next_state, &engine).unwrap(),
            Bounds::exact(backing)
        );
        assert_eq!(
            sale.reserve,
            Bounds::exact(nn(150).checked_sub(backing).unwrap())
        );
        let buy = calculator
            .quote_issue(&sale.next_state, pos(1), &engine)
            .unwrap();
        assert_eq!(buy.reserve, Bounds::exact(rate.checked_add(nn(4)).unwrap()));
        assert_eq!(
            buy.next_state.segmentation(),
            sale.next_state.segmentation()
        );
    }

    let issued = calculator
        .quote_issue(&MarketState::launch(nn(1)), pos(1), &engine)
        .unwrap();
    let exercised = calculator
        .quote_issue_at_floor(&issued.next_state, pos(100))
        .unwrap();
    let sale = calculator
        .quote_redeem(&exercised.next_state, pos(50), &engine)
        .unwrap();
    // One unit unwinds the original 5 reserve; the other 49 sell at the floor.
    assert_eq!(sale.reserve, Bounds::exact(nn(54)));
    assert_eq!(sale.next_state.accounting(), Accounting::new(nn(0), nn(51)));
    assert_eq!(
        calculator.backing(&sale.next_state, &engine).unwrap(),
        Bounds::exact(nn(51))
    );

    let burned = calculator.quote_redeem_at_floor(&raised(), pos(3)).unwrap();
    let exit = calculator
        .quote_redeem(&burned.next_state, pos(3), &engine)
        .unwrap();
    // Zero circulation can retain positive progress after an earlier floor burn.
    assert_eq!(exit.next_state.accounting(), Accounting::new(nn(3), nn(0)));
    assert_eq!(exit.reserve, Bounds::exact(nn(102)));
    assert_eq!(
        calculator.backing(&exit.next_state, &engine).unwrap(),
        Bounds::exact(nn(0))
    );
}

#[test]
fn rounded_contraction_funds_the_retained_ramp_before_paying_the_seller() {
    type Tenths = I64Scaled<10>;
    let n = |raw| NonNeg::new(Tenths::from_scaled_i64(raw)).unwrap();
    let engine = LocalCharacteristic::new(LinearCharacteristic::new(Pos::try_from(n(13)).unwrap()));
    let calculator = AvmCalculator::new(AboveOne::new(n(15).get()).unwrap());
    let state = MarketState::new(
        Segmentation::new(n(10), n(10), n(10), n(17).get()).unwrap(),
        Accounting::new(n(10), n(10)),
    )
    .unwrap();
    let sale = calculator
        .quote_redeem(&state, Pos::try_from(n(7)).unwrap(), &engine)
        .unwrap();
    let offset = sale.next_state.segmentation().main_offset().to_scaled_i64();
    assert_eq!(offset, 13);
    // Exact ideal offset is 1.245. Stored offset 1.3 lifts the retained ramp
    // by 0.055, costing 0.0165 reserve. Before the sale backing is 2.025;
    // after the sale it is 0.41925. All comparisons use independent integers.
    let payout = sale.reserve.lower().get().to_scaled_i64();
    assert!(payout > 0);
    assert!(payout * 10_000 + 41_925 <= 202_500);
    assert_eq!(sale.next_state.accounting(), Accounting::new(n(3), n(3)));
    let buy = calculator
        .quote_issue(&sale.next_state, Pos::try_from(n(1)).unwrap(), &engine)
        .unwrap();
    // The new buy follows main: integral(1.3 + 1.3x, 0.3, 0.1) = 0.1755.
    assert!(buy.reserve.lower().get().to_scaled_i64() * 1_000 <= 1_755);
    assert!(buy.reserve.upper().get().to_scaled_i64() * 1_000 >= 1_755);

    // On this deliberately coarse grid, re-anchoring a long retained ramp
    // cannot be certified as funded by a tiny sale. Do not return a new state.
    let engine = LocalCharacteristic::new(LinearCharacteristic::new(Pos::try_from(n(1)).unwrap()));
    let state = MarketState::new(
        Segmentation::new(n(10), n(1_000), n(1_000), n(60).get()).unwrap(),
        Accounting::new(n(1_000), n(1_000)),
    )
    .unwrap();
    assert_eq!(
        calculator
            .quote_redeem(&state, Pos::try_from(n(1)).unwrap(), &engine)
            .err(),
        Some(Error::ContractionPrecisionLoss),
    );
}

#[derive(Default)]
struct ObservedEngine {
    batches: RefCell<Vec<Vec<SpanRequest<N>>>>,
}

impl ObservedEngine {
    fn take_batch(&self) -> Vec<SpanRequest<N>> {
        let mut batches = self.batches.take();
        assert_eq!(batches.len(), 1);
        batches.pop().unwrap()
    }
}

impl CharacteristicEngine<N> for ObservedEngine {
    fn spans(
        &self,
        requests: &[SpanRequest<N>],
        responses: &mut [SpanResponse<N>],
    ) -> CharacteristicEngineResult<()> {
        self.batches.borrow_mut().push(requests.to_vec());
        engine().spans(requests, responses)
    }
}

#[test]
fn market_calculations_batch_engine_calls_and_keep_floor_operations_local() {
    let calculator = calculator();
    let engine = ObservedEngine::default();
    let burn = calculator.quote_redeem_at_floor(&raised(), pos(3)).unwrap();
    let exercise = calculator
        .quote_issue_at_floor(&burn.next_state, pos(2))
        .unwrap();
    assert_eq!(burn.reserve, Bounds::exact(nn(48)));
    assert_eq!(exercise.reserve, Bounds::exact(nn(32)));
    assert_eq!(burn.next_state.accounting(), Accounting::new(nn(6), nn(3)));
    assert_eq!(
        exercise.next_state.accounting(),
        Accounting::new(nn(6), nn(5))
    );
    assert!(engine.batches.borrow().is_empty());
    assert_eq!(
        calculator
            .quote_redeem_at_floor(&burn.next_state, pos(1))
            .err(),
        Some(Error::InsufficientFloorTail)
    );
    for (state, backing) in [(burn.next_state, 102), (exercise.next_state, 134)] {
        assert_eq!(state.segmentation(), raised().segmentation());
        assert_eq!(
            calculator.price(&state, &engine).unwrap(),
            Bounds::exact(nn(49))
        );
        // Main evaluation is independent of all ramp and floor parameters.
        assert_eq!(
            engine.take_batch(),
            vec![SpanRequest {
                lo: nn(6),
                width: nn(0)
            }]
        );
        assert_eq!(
            calculator.backing(&state, &engine).unwrap(),
            Bounds::exact(nn(backing))
        );
        engine.take_batch();
    }
    calculator.quote_issue(&raised(), pos(1), &engine).unwrap();
    assert_eq!(
        engine.take_batch(),
        vec![SpanRequest {
            lo: nn(6),
            width: nn(1)
        }]
    );
    for amount in [2, 5] {
        calculator
            .quote_redeem(&raised(), pos(amount), &engine)
            .unwrap();
        // The sale and its contraction share the same synchronous engine batch.
        let batch = engine.take_batch();
        for (index, request) in batch.iter().enumerate() {
            assert!(!batch[..index].contains(request));
        }
    }
    let initial =
        MarketState::new(Segmentation::launch(nn(1)), Accounting::new(nn(6), nn(6))).unwrap();
    calculator
        .verify_floor_raise(&initial, proposal(), FloorRaisePolicy::exact(), &engine)
        .unwrap();
    engine.take_batch();
}

#[test]
fn floor_raise_limits_are_signed_and_include_all_circulating_backing() {
    let calculator = calculator();
    let engine = engine();
    let issued = calculator
        .quote_issue(&MarketState::launch(nn(1)), pos(6), &engine)
        .unwrap()
        .next_state;
    let with_options = calculator
        .quote_issue_at_floor(&issued, pos(2))
        .unwrap()
        .next_state;
    // The net-issued area matches, but the extra two tokens require 30 more reserve.
    for allowance in [N::ZERO, nn(29).get(), negative(nn(30))] {
        assert_eq!(
            calculator
                .verify_floor_raise(
                    &with_options,
                    proposal(),
                    FloorRaisePolicy::new(nn(0), allowance),
                    &engine,
                )
                .err(),
            Some(Error::AreaChangeOutsideTolerance)
        );
    }
    let funded = calculator
        .verify_floor_raise(
            &with_options,
            proposal(),
            FloorRaisePolicy::new(nn(0), nn(30).get()),
            &engine,
        )
        .unwrap();
    assert_eq!(funded.backing_change, Bounds::exact(nn(30).get()));
    assert_eq!(funded.next_state.segmentation().main_offset(), nn(1).get());

    let surplus = FloorRaiseProposal {
        ramp_end: fraction(185, 32),
        ramp_width: fraction(5, 2),
        ..proposal()
    };
    for allowance in [N::ZERO, negative(nn(7)), nn(8).get()] {
        assert_eq!(
            calculator
                .verify_floor_raise(
                    &issued,
                    surplus,
                    FloorRaisePolicy::new(nn(0), allowance),
                    &engine,
                )
                .err(),
            Some(Error::AreaChangeOutsideTolerance)
        );
    }
    let accepted = calculator
        .verify_floor_raise(
            &issued,
            surplus,
            FloorRaisePolicy::new(nn(0), negative(nn(8))),
            &engine,
        )
        .unwrap();
    assert_eq!(
        accepted.next_state.segmentation().main_offset(),
        nn(1).get()
    );
    assert_eq!(
        accepted.backing_change,
        Bounds::exact(negative(fraction(2025, 256)))
    );

    // A slightly lower proposed floor leaves an upward gap at ramp start.
    // It changes only the floor rectangle, never the independently stored main.
    let gap_candidate = FloorRaiseProposal {
        floor_increase_by: Pos::try_from(fraction(239, 16)).unwrap(),
        ..proposal()
    };
    assert_eq!(
        calculator
            .verify_floor_raise(
                &issued,
                gap_candidate,
                FloorRaisePolicy::new(fraction(1, 32), negative(nn(1))),
                &engine,
            )
            .err(),
        Some(Error::FloorJoinOutsideTolerance)
    );
    let accepted = calculator
        .verify_floor_raise(
            &issued,
            gap_candidate,
            FloorRaisePolicy::new(fraction(1, 16), negative(fraction(3, 16))),
            &engine,
        )
        .unwrap();
    assert_eq!(
        accepted.backing_change,
        Bounds::exact(negative(fraction(3, 16)))
    );
    let before = ScheduleCalculator::new(issued.segmentation(), scalar());
    let after = ScheduleCalculator::new(accepted.next_state.segmentation(), scalar());
    for x in [nn(5), nn(6), fraction(103, 16), nn(100)] {
        assert_eq!(before.y(x, &engine).unwrap(), after.y(x, &engine).unwrap());
        assert_eq!(
            before.integrate(x, fraction(13, 16), &engine).unwrap(),
            after.integrate(x, fraction(13, 16), &engine).unwrap()
        );
    }

    // A floor above the ramp start would make a downward jump; a generous
    // tolerance must not make that candidate admissible.
    assert_eq!(
        calculator
            .verify_floor_raise(
                &issued,
                FloorRaiseProposal {
                    floor_increase_by: Pos::try_from(fraction(241, 16)).unwrap(),
                    ..proposal()
                },
                FloorRaisePolicy::new(nn(1), nn(1).get()),
                &engine,
            )
            .err(),
        Some(Error::FloorJoinOutsideTolerance)
    );
}

#[test]
fn floor_raise_residuals_match_whole_schedule_accounting() {
    let calculator = AvmCalculator::new(AboveOne::new(nn(2).get()).unwrap());
    let engine = LocalCharacteristic::new(LinearCharacteristic::new(pos(1)));
    for (old_floor, old_start, old_end, floor, start, end, progress, supply) in [
        (4, 4, 6, 6, 5, 6, 8, 8),         // Funded narrowing, fixed ramp end.
        (14, 14, 21, 20, 18, 23, 28, 14), // Exact-area narrowing after a floor burn.
        (2, 2, 3, 3, 3, 4, 5, 5),         // Both boundaries advance; backing grows.
        (2, 2, 3, 3, 4, 6, 6, 6),         // Both boundaries advance; backing shrinks.
        (2, 2, 3, 3, 3, 4, 7, 9),         // Extra circulation adds floor backing.
        (2, 2, 3, 3, 3, 4, 7, 5),         // Floor burns remove that extra rectangle.
        (2, 2, 2, 3, 3, 3, 5, 4),         // Both ramps have zero width.
    ] {
        let offset = N::checked_from_i128(old_floor + old_end - 2 * old_start).unwrap();
        let old =
            Segmentation::new(nn(old_floor), nn(old_end), nn(old_end - old_start), offset).unwrap();
        let state = MarketState::new(old, Accounting::new(nn(progress), nn(supply))).unwrap();
        let candidate = FloorRaiseProposal {
            floor_increase_by: Pos::try_from(nn(floor - old_floor)).unwrap(),
            ramp_end: nn(end),
            ramp_width: nn(end - start),
        };
        // Independent whole-curve antiderivative for f(x)=x, k=2. All inputs
        // are integers, so these doubled backing values have no rounding.
        let backing_twice = |floor: i128, start: i128, end: i128| {
            2 * floor * supply + 2 * (progress - start).pow(2) - (progress - end).pow(2)
        };
        let area_change_twice =
            backing_twice(floor, start, end) - backing_twice(old_floor, old_start, old_end);
        let area_change = N::checked_from_ratio_round(
            area_change_twice,
            2u128.try_into().unwrap(),
            DirectedRounding::Floor,
        )
        .unwrap();
        let raise = calculator
            .verify_floor_raise(
                &state,
                candidate,
                FloorRaisePolicy::new(nn(0), area_change),
                &engine,
            )
            .unwrap();
        assert_eq!(raise.next_state.segmentation().main_offset(), offset);
        assert_eq!(raise.backing_change, Bounds::exact(area_change));
        assert_eq!(
            raise.backing,
            Bounds::exact(fraction(backing_twice(floor, start, end), 2)),
        );
        assert_eq!(
            calculator.price(&raise.next_state, &engine).unwrap(),
            Bounds::exact(nn(progress + floor + end - 2 * start))
        );
        if area_change_twice != 0 {
            let tighter = N::checked_from_ratio_round(
                area_change_twice - area_change_twice.signum(),
                2u128.try_into().unwrap(),
                DirectedRounding::Floor,
            )
            .unwrap();
            assert_eq!(
                calculator
                    .verify_floor_raise(
                        &state,
                        candidate,
                        FloorRaisePolicy::new(nn(0), tighter),
                        &engine,
                    )
                    .err(),
                Some(Error::AreaChangeOutsideTolerance)
            );
        }
    }
}

#[test]
fn invalid_market_geometry_and_unrepresentable_quotes_return_errors() {
    assert_eq!(
        Segmentation::new(nn(1), nn(2), nn(3), nn(1).get()),
        Err(Error::RampWidthExceedsEnd)
    );
    assert_eq!(
        MarketState::new(raised().segmentation(), Accounting::new(nn(4), nn(6))),
        Err(Error::RampEndBeyondProgress)
    );
    assert_eq!(
        MarketState::new(raised().segmentation(), Accounting::new(nn(6), nn(2))),
        Err(Error::InsufficientCirculation)
    );
    let engine = engine();
    let calculator = calculator();
    assert_eq!(
        calculator.quote_redeem(&raised(), pos(7), &engine).err(),
        Some(Error::InsufficientCirculation)
    );
    let max = NonNeg::new(N::MAX).unwrap();
    let at_limit =
        MarketState::new(Segmentation::launch(nn(1)), Accounting::new(max, max)).unwrap();
    assert_eq!(
        calculator.quote_issue_at_floor(&at_limit, pos(1)).err(),
        Some(Error::ArithmeticFailure)
    );
    assert_eq!(
        calculator.quote_issue(&at_limit, pos(1), &engine).err(),
        Some(Error::ArithmeticFailure)
    );

    let zero = Bounds::exact(nn(0));
    let mut responses = [SpanResponse {
        y_at_lo: zero,
        y_at_hi: zero,
        area: zero,
    }];
    let request = SpanRequest {
        lo: nn(0),
        width: nn(1),
    };
    assert_eq!(
        engine.spans(&[request], &mut []),
        Err(CharacteristicEngineError::InvalidBatch)
    );
    assert_eq!(
        engine.spans(
            &[SpanRequest {
                lo: max,
                width: nn(1)
            }],
            &mut responses[..1]
        ),
        Err(CharacteristicEngineError::Calculation(
            CharacteristicError::ArithmeticFailure
        ))
    );

    let free = calculator
        .quote_issue_at_floor(&MarketState::launch(nn(0)), pos(1))
        .unwrap();
    assert_eq!(free.reserve, Bounds::exact(nn(0)));
    assert_eq!(free.next_state.accounting(), Accounting::new(nn(0), nn(1)));
}

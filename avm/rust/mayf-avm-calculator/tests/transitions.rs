use std::cell::RefCell;

use mayf_avm_calculator::{
    Accounting, AvmCalculator, CharacteristicEngine, CharacteristicEngineError,
    CharacteristicEngineResult, Error, FloorRaisePolicy, FloorRaiseProposal, LocalCharacteristic,
    MarketState, ScheduleCalculator, Segmentation, SpanRequest, SpanResponse,
};
use mayf_characteristic::{AffineTransform, Characteristic};
use mayf_characteristic_linear_driver::{LinearCalculator, LinearCharacteristic};
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{
    AboveOne, Bounds, DirectedRounding, FixedNumber, NonNeg, Pos, SignedFixed,
};

use mayf_monotone::MonotoneCalculatorError;

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

fn linear() -> LinearCharacteristic<N> {
    LinearCharacteristic::new(pos(8))
}

fn engine() -> LocalCharacteristic<LinearCharacteristic<N>> {
    LocalCharacteristic::new(linear())
}

fn calculator() -> AvmCalculator<N> {
    AvmCalculator::new(scalar())
}

fn negative(value: NonNeg<N>) -> N {
    value.get().checked_neg().unwrap()
}

fn proposal() -> FloorRaiseProposal<N> {
    FloorRaiseProposal {
        floor: nn(16),
        ramp_end: nn(5),
        ramp_width: nn(2),
    }
}

fn raised() -> MarketState<N> {
    MarketState::new(
        Segmentation::new(nn(16), nn(5), nn(2)).unwrap(),
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

struct OriginOnly;

impl Characteristic<N> for OriginOnly {
    type Calculator = LinearCalculator<N>;

    fn calculator(&self, affine: AffineTransform<N>) -> Self::Calculator {
        assert!(
            affine.is_identity(),
            "only a neutral characteristic view is requested"
        );
        linear().calculator(affine)
    }
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
        LocalCharacteristic::new(OriginOnly).spans(requests, responses)
    }
}

#[test]
fn market_calculations_use_one_neutral_batch_and_floor_operations_use_none() {
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
    assert_eq!(
        calculator
            .quote_redeem_at_floor(&burn.next_state, pos(1))
            .err(),
        Some(Error::InsufficientFloorTail)
    );
    let contracted = raised().segmentation().contract_to(nn(4)).unwrap();
    assert!(engine.batches.borrow().is_empty());

    // Collapsed ramps move both boundaries over the same interval only once.
    let collapsed = MarketState::new(
        Segmentation::new(nn(8), nn(1), nn(0)).unwrap(),
        Accounting::new(nn(3), nn(3)),
    )
    .unwrap();
    calculator
        .verify_floor_raise(
            &collapsed,
            FloorRaiseProposal {
                floor: nn(16),
                ramp_end: nn(2),
                ramp_width: nn(0),
            },
            FloorRaisePolicy::new(nn(0), nn(12).get()),
            &engine,
        )
        .unwrap();
    assert_eq!(
        engine.take_batch(),
        vec![SpanRequest {
            lo: nn(1),
            width: nn(1)
        }]
    );
    for (state, backing) in [(burn.next_state, 102), (exercise.next_state, 134)] {
        assert_eq!(state.segmentation(), raised().segmentation());
        assert_eq!(
            calculator.price(&state, &engine).unwrap(),
            Bounds::exact(nn(49))
        );
        assert_eq!(engine.take_batch().len(), 2);
        assert_eq!(
            calculator.backing(&state, &engine).unwrap(),
            Bounds::exact(nn(backing))
        );
        assert_eq!(engine.take_batch().len(), 2);
    }

    calculator.quote_issue(&raised(), pos(1), &engine).unwrap();
    let batch = engine.take_batch();
    assert_eq!(batch.len(), 2);
    assert!(batch.iter().any(|s| s.lo == nn(6) && s.width == nn(1)));

    // A partial-ramp sale needs its two redeemed intervals and the ramp's
    // starting price, not another integral over the entire original ramp.
    calculator.quote_redeem(&raised(), pos(2), &engine).unwrap();
    let batch = engine.take_batch();
    assert_eq!(batch.len(), 3);
    for (lo, width) in [(4, 1), (3, 0), (5, 1)] {
        assert!(batch.contains(&SpanRequest {
            lo: nn(lo),
            width: nn(width)
        }));
    }
    // Reaching the floor redeems the full ramp, which already supplies its anchor.
    calculator.quote_redeem(&raised(), pos(5), &engine).unwrap();
    assert_eq!(engine.take_batch().len(), 2);
    let initial =
        MarketState::new(Segmentation::launch(nn(1)), Accounting::new(nn(6), nn(6))).unwrap();
    calculator
        .verify_floor_raise(&initial, proposal(), FloorRaisePolicy::exact(), &engine)
        .unwrap();
    let batch = engine.take_batch();
    assert_eq!(batch.len(), 2);
    for width in [3, 5] {
        assert!(batch.contains(&SpanRequest {
            lo: nn(0),
            width: nn(width)
        }));
    }

    // An unchanged ramp end contributes no change and requires no point query.
    calculator
        .verify_floor_raise(
            &raised(),
            FloorRaiseProposal {
                floor: fraction(57, 2),
                ramp_end: nn(5),
                ramp_width: nn(1),
            },
            FloorRaisePolicy::new(nn(0), fraction(175, 4).get()),
            &engine,
        )
        .unwrap();
    assert_eq!(
        engine.take_batch(),
        vec![SpanRequest {
            lo: nn(3),
            width: nn(1)
        }]
    );

    // With both boundaries fixed, the permitted vertical change is a rectangle.
    calculator
        .verify_floor_raise(
            &raised(),
            FloorRaiseProposal {
                floor: nn(17),
                ..proposal()
            },
            FloorRaisePolicy::new(nn(1), nn(6).get()),
            &engine,
        )
        .unwrap();
    assert!(engine.batches.borrow().is_empty());

    let schedule = ScheduleCalculator::new(contracted, scalar());
    assert_eq!(
        schedule.y(nn(4), &engine).unwrap(),
        Bounds::exact(fraction(57, 2))
    );
    assert_eq!(engine.take_batch().len(), 1);
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
    assert_eq!(funded.main_change, Bounds::exact(N::ZERO));

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
    assert_eq!(accepted.main_change, Bounds::exact(N::ZERO));
    assert_eq!(
        accepted.backing_change,
        Bounds::exact(negative(fraction(2025, 256)))
    );

    let lowered_main = FloorRaiseProposal {
        floor: fraction(255, 16),
        ..proposal()
    };
    assert_eq!(
        calculator
            .verify_floor_raise(
                &issued,
                lowered_main,
                FloorRaisePolicy::new(fraction(1, 32), negative(nn(1))),
                &engine,
            )
            .err(),
        Some(Error::MainCurveChanged)
    );
    let accepted = calculator
        .verify_floor_raise(
            &issued,
            lowered_main,
            FloorRaisePolicy::new(fraction(1, 16), negative(fraction(3, 8))),
            &engine,
        )
        .unwrap();
    assert_eq!(
        accepted.backing_change,
        Bounds::exact(negative(fraction(3, 8)))
    );

    // Overlapping price bounds do not certify an unchanged main.
    let uncertain = FloorRaiseProposal {
        floor: NonNeg::new(nn(16).get().checked_sub(N::from_scaled_i128(12)).unwrap()).unwrap(),
        ramp_width: NonNeg::new(nn(2).get().checked_add(N::QUANTUM).unwrap()).unwrap(),
        ..proposal()
    };
    let candidate = ScheduleCalculator::new(
        Segmentation::new(uncertain.floor, uncertain.ramp_end, uncertain.ramp_width).unwrap(),
        scalar(),
    );
    let endpoint = candidate.y(nn(5), &engine).unwrap();
    assert!(endpoint.contains(nn(41)) && !endpoint.is_exact());
    assert_eq!(
        calculator
            .verify_floor_raise(&issued, uncertain, FloorRaisePolicy::exact(), &engine)
            .err(),
        Some(Error::MainCurveChanged)
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
        (2, 2, 3, 4, 3, 4, 7, 9),         // Permitted positive main shift, supply > progress.
        (2, 2, 3, 3, 4, 5, 7, 5),         // Permitted negative main shift, supply < progress.
        (2, 2, 2, 3, 3, 3, 5, 4),         // Both ramps have zero width.
        (2, 2, 3, 3, 2, 3, 5, 4),         // Neither boundary moves.
    ] {
        let old = Segmentation::new(nn(old_floor), nn(old_end), nn(old_end - old_start)).unwrap();
        let state = MarketState::new(old, Accounting::new(nn(progress), nn(supply))).unwrap();
        let candidate = FloorRaiseProposal {
            floor: nn(floor),
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
        let main_change = floor + end - 2 * start - (old_floor + old_end - 2 * old_start);
        let raise = calculator
            .verify_floor_raise(
                &state,
                candidate,
                FloorRaisePolicy::new(nn(main_change.abs()), area_change),
                &engine,
            )
            .unwrap();
        assert_eq!(
            raise.main_change,
            Bounds::exact(N::checked_from_i128(main_change).unwrap())
        );
        assert_eq!(raise.backing_change, Bounds::exact(area_change));
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
                        FloorRaisePolicy::new(nn(main_change.abs()), tighter),
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
        Segmentation::new(nn(1), nn(2), nn(3)),
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
            MonotoneCalculatorError::ArithmeticFailure
        ))
    );

    let free = calculator
        .quote_issue_at_floor(&MarketState::launch(nn(0)), pos(1))
        .unwrap();
    assert_eq!(free.reserve, Bounds::exact(nn(0)));
    assert_eq!(free.next_state.accounting(), Accounting::new(nn(0), nn(1)));
}

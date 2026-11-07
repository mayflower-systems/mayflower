use std::cell::Cell;

use mayf_avm_calculator::{
    Accounting, AvmCalculator, CharacteristicEngine, CharacteristicEngineResult,
    Error as CalculatorError, LocalCharacteristic, MarketState, Segmentation, SpanRequest,
    SpanResponse,
};
use mayf_avm_prog::{
    actions::{
        FloorRaiseError, RaiseFloorFromSurplusArgs, RaiseFloorPreservingAreaArgs,
        raise_floor_from_surplus, raise_floor_preserving_area,
    },
    state::{FeeRates, Market},
};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{
    AboveOne, Bounds, DirectedRounding, FixedNumber, NonNeg, Pos, SignedFixed,
};
use mayf_micro_basis_points::MicroBasisPoints;

type N = I64F64;

fn n(numerator: i128, denominator: i128) -> NonNeg<N> {
    NonNeg::new(
        N::checked_from_ratio_round(
            numerator,
            u128::try_from(denominator).unwrap().try_into().unwrap(),
            DirectedRounding::Floor,
        )
        .unwrap(),
    )
    .unwrap()
}

fn p(numerator: i128, denominator: i128) -> Pos<N> {
    Pos::try_from(n(numerator, denominator)).unwrap()
}

#[derive(Default)]
struct Engine {
    calls: Cell<usize>,
    widen_area: bool,
}

impl CharacteristicEngine<N> for Engine {
    fn spans(
        &self,
        requests: &[SpanRequest<N>],
        responses: &mut [SpanResponse<N>],
    ) -> CharacteristicEngineResult<()> {
        self.calls.set(self.calls.get() + 1);
        LocalCharacteristic::new(LinearCharacteristic::new(p(8, 1))).spans(requests, responses)?;
        if self.widen_area {
            for (request, response) in requests.iter().zip(responses) {
                if request.width != NonNeg::zero() {
                    response.area = Bounds::new(
                        response.area.lower(),
                        response
                            .area
                            .upper()
                            .checked_add(NonNeg::new(N::QUANTUM).unwrap())
                            .unwrap(),
                    )
                    .unwrap();
                }
            }
        }
        Ok(())
    }
}

fn market() -> Market<N> {
    // Main is 1 + 8x; at progress 6 it requires 150 reserve.
    Market {
        calculator_state: MarketState::new(
            Segmentation::launch(n(1, 1)),
            Accounting::new(n(6, 1), n(6, 1)),
        )
        .unwrap(),
        ramp_scalar: AboveOne::new(n(25, 16).get()).unwrap(),
        reserve_held: n(100, 1),
        reserve_advanced: n(50, 1),
        fee_rates: FeeRates {
            issue_fee: MicroBasisPoints::from_percent(1).unwrap(),
            redeem_fee: MicroBasisPoints::from_percent(2).unwrap(),
            exercise_fee: MicroBasisPoints::from_percent(3).unwrap(),
            reserve_access_fee: MicroBasisPoints::from_percent(4).unwrap(),
        },
        protocol_fee_rate: MicroBasisPoints::from_percent(25).unwrap(),
    }
}

fn preserving(market: Market<N>) -> RaiseFloorPreservingAreaArgs<N> {
    RaiseFloorPreservingAreaArgs {
        floor_increase_by: p(15, 1),
        new_ramp_end: n(5, 1),
        new_ramp_width: n(2, 1),
        max_floor_join_gap: n(0, 1),
        max_area_decrease: n(0, 1),
        market,
    }
}

fn surplus(market: Market<N>) -> RaiseFloorFromSurplusArgs<N> {
    // A = 13/4, E = 5, F = 153/8; area grows by 625/64.
    RaiseFloorFromSurplusArgs {
        floor_increase_by: p(145, 8),
        new_ramp_end: n(5, 1),
        new_ramp_width: n(7, 4),
        max_floor_join_gap: n(0, 1),
        max_area_increase: n(625, 64),
        market,
    }
}

#[test]
fn exact_raise_preserves_main_accounting_and_reserves_in_one_engine_call() {
    let engine = Engine::default();
    let original = market();
    let receipt = raise_floor_preserving_area(preserving(original), &engine).unwrap();
    assert_eq!(engine.calls.get(), 1);
    assert_eq!(receipt.backing_change, Bounds::exact(n(0, 1).get()));
    assert_eq!(
        receipt.market_next,
        Market {
            calculator_state: MarketState::new(
                Segmentation::new(n(16, 1), n(5, 1), n(2, 1), n(1, 1).get()).unwrap(),
                original.calculator_state.accounting(),
            )
            .unwrap(),
            ..original
        }
    );
    let calculator = AvmCalculator::new(original.ramp_scalar);
    assert_eq!(
        calculator.price(&receipt.market_next.calculator_state, &engine),
        calculator.price(&original.calculator_state, &engine)
    );
}

#[test]
fn directional_budgets_accept_the_limit_and_reject_the_opposite_direction() {
    let engine = Engine::default();
    let loss = n(575, 64);
    let shrink = RaiseFloorPreservingAreaArgs {
        // A = 11/4; this candidate loses exactly 575/64 reserve.
        floor_increase_by: p(95, 8),
        new_ramp_width: n(9, 4),
        max_area_decrease: loss,
        ..preserving(market())
    };
    assert_eq!(
        raise_floor_preserving_area(shrink, &engine)
            .unwrap()
            .backing_change,
        Bounds::exact(loss.get().checked_neg().unwrap())
    );
    let quantum = NonNeg::new(N::QUANTUM).unwrap();
    let outside = FloorRaiseError::Calculator(CalculatorError::AreaChangeOutsideTolerance);
    assert_eq!(
        raise_floor_preserving_area(
            RaiseFloorPreservingAreaArgs {
                max_area_decrease: loss.checked_sub(quantum).unwrap(),
                ..shrink
            },
            &engine,
        ),
        Err(outside)
    );
    assert_eq!(
        raise_floor_from_surplus(
            RaiseFloorFromSurplusArgs {
                floor_increase_by: shrink.floor_increase_by,
                new_ramp_width: shrink.new_ramp_width,
                ..surplus(market())
            },
            &engine,
        ),
        Err(outside)
    );

    let growth = surplus(Market {
        reserve_held: n(100, 1).checked_add(n(625, 64)).unwrap(),
        ..market()
    });
    let before = engine.calls.get();
    let receipt = raise_floor_from_surplus(growth, &engine).unwrap();
    assert_eq!(engine.calls.get() - before, 1);
    assert_eq!(receipt.backing_change, Bounds::exact(n(625, 64).get()));
    assert_eq!(receipt.market_next.reserve_held, growth.market.reserve_held);
    assert_eq!(
        receipt.market_next.reserve_advanced,
        growth.market.reserve_advanced
    );
    assert_eq!(
        raise_floor_from_surplus(
            RaiseFloorFromSurplusArgs {
                max_area_increase: growth.max_area_increase.checked_sub(quantum).unwrap(),
                ..growth
            },
            &engine,
        ),
        Err(outside)
    );
    assert_eq!(
        raise_floor_preserving_area(
            RaiseFloorPreservingAreaArgs {
                floor_increase_by: growth.floor_increase_by,
                new_ramp_width: growth.new_ramp_width,
                max_area_decrease: n(100, 1),
                ..preserving(growth.market)
            },
            &engine,
        ),
        Err(outside)
    );
}

#[test]
fn surplus_requires_actual_backing_and_accounts_for_outstanding_advances() {
    let engine = Engine::default();
    assert_eq!(
        raise_floor_from_surplus(surplus(market()), &engine),
        Err(FloorRaiseError::InsufficientReserves)
    );
    let total = n(150, 1).checked_add(n(625, 64)).unwrap();
    for advanced in [n(0, 1), n(50, 1), total] {
        let args = surplus(Market {
            reserve_held: total.checked_sub(advanced).unwrap(),
            reserve_advanced: advanced,
            ..market()
        });
        raise_floor_from_surplus(args, &engine).unwrap();
    }

    // Funding uses the upper backing bound, even when the nominal area fits.
    let uncertain = Engine {
        widen_area: true,
        ..Engine::default()
    };
    assert_eq!(
        raise_floor_from_surplus(
            RaiseFloorFromSurplusArgs {
                max_area_increase: n(10, 1),
                ..surplus(Market {
                    reserve_held: total,
                    reserve_advanced: n(0, 1),
                    ..market()
                })
            },
            &uncertain,
        ),
        Err(FloorRaiseError::InsufficientReserves)
    );
    // A straddling enclosure does not prove that an ordinary raise cannot grow.
    assert_eq!(
        raise_floor_preserving_area(
            RaiseFloorPreservingAreaArgs {
                max_area_decrease: n(1, 1),
                ..preserving(market())
            },
            &uncertain,
        ),
        Err(FloorRaiseError::Calculator(
            CalculatorError::AreaChangeOutsideTolerance
        ))
    );
}

#[test]
fn floor_tail_changes_are_part_of_the_authorized_reserve_budget() {
    let engine = Engine::default();
    // The progress-axis areas are equal for F=16,E=5,W=2 and F=31,E=10,W=4.
    // With progress 12, two burned/exercised floor tokens change backing by -/+30.
    for circulation in [10, 12, 14] {
        let market = Market {
            calculator_state: MarketState::new(
                Segmentation::new(n(16, 1), n(5, 1), n(2, 1), n(1, 1).get()).unwrap(),
                Accounting::new(n(12, 1), n(circulation, 1)),
            )
            .unwrap(),
            reserve_held: n(1_000, 1),
            ..market()
        };
        let receipt = if circulation <= 12 {
            raise_floor_preserving_area(
                RaiseFloorPreservingAreaArgs {
                    new_ramp_end: n(10, 1),
                    new_ramp_width: n(4, 1),
                    max_area_decrease: n(30, 1),
                    ..preserving(market)
                },
                &engine,
            )
        } else {
            raise_floor_from_surplus(
                RaiseFloorFromSurplusArgs {
                    floor_increase_by: p(15, 1),
                    new_ramp_end: n(10, 1),
                    new_ramp_width: n(4, 1),
                    max_area_increase: n(30, 1),
                    ..surplus(market)
                },
                &engine,
            )
        }
        .unwrap();
        assert_eq!(
            receipt.backing_change,
            Bounds::exact(N::checked_from_i128((circulation - 12) * 15).unwrap())
        );
    }
}

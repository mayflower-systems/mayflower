use std::{cell::RefCell, rc::Rc};

use mayf_avm_calculator::{
    Accounting, AffineTransform, AvmCalculator, Characteristic, CharacteristicEngine, Error,
    FloorRaisePolicy, FloorRaiseProposal, MarketState, Segmentation, compile_schedule,
};
use mayf_characteristic_linear_driver::{LinearCalculator, LinearCharacteristic};
use mayf_fixed_drivers::I64F64;
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

fn linear() -> LinearCharacteristic<N> {
    LinearCharacteristic::new(pos(8))
}

fn engine() -> CharacteristicEngine<LinearCharacteristic<N>, N> {
    CharacteristicEngine::new(linear(), scalar())
}

fn proposal() -> FloorRaiseProposal<N> {
    FloorRaiseProposal {
        floor: pos(16),
        ramp_end: nn(5),
        ramp_width: nn(2),
    }
}

fn raised() -> MarketState<N> {
    MarketState::new(
        Segmentation::new(pos(16), nn(5), nn(2)).unwrap(),
        Accounting::new(nn(6), nn(6)),
    )
    .unwrap()
}

#[test]
fn ordinary_sales_retain_the_prefix_and_resume_buying_on_main() {
    let engine = engine();
    for (remaining, width, rate, backing) in [
        (5, 2, nn(41), nn(105)),
        (4, 1, fraction(57, 2), fraction(281, 4)),
        (3, 0, nn(16), nn(48)),
        (2, 0, nn(16), nn(32)),
        (0, 0, nn(16), nn(0)),
    ] {
        let sale = engine.quote_redeem(&raised(), pos(6 - remaining)).unwrap();
        assert_eq!(
            sale.next_state.accounting(),
            Accounting::new(nn(remaining.into()), nn(remaining.into()))
        );
        assert_eq!(
            sale.next_state.segmentation().ramp_end(),
            nn(remaining.into())
        );
        assert_eq!(sale.next_state.segmentation().ramp_width(), nn(width));
        assert_eq!(engine.price(&sale.next_state).unwrap(), Bounds::exact(rate));
        assert_eq!(
            engine.backing(&sale.next_state).unwrap(),
            Bounds::exact(backing)
        );
        assert_eq!(
            sale.reserve,
            Bounds::exact(nn(150).checked_sub(backing).unwrap())
        );
        let buy = engine.quote_issue(&sale.next_state, pos(1)).unwrap();
        assert_eq!(buy.reserve, Bounds::exact(rate.checked_add(nn(4)).unwrap()));
        assert_eq!(
            buy.next_state.segmentation(),
            sale.next_state.segmentation()
        );
    }

    let issued = engine
        .quote_issue(&MarketState::launch(pos(1)), pos(1))
        .unwrap();
    let exercised = engine
        .quote_issue_at_floor(&issued.next_state, pos(100))
        .unwrap();
    let sale = engine.quote_redeem(&exercised.next_state, pos(50)).unwrap();
    // One unit unwinds the original 5 reserve; the other 49 sell at the floor.
    assert_eq!(sale.reserve, Bounds::exact(nn(54)));
    assert_eq!(sale.next_state.accounting(), Accounting::new(nn(0), nn(51)));
    assert_eq!(
        engine.backing(&sale.next_state).unwrap(),
        Bounds::exact(nn(51))
    );

    let burned = engine.quote_redeem_at_floor(&raised(), pos(3)).unwrap();
    let exit = engine.quote_redeem(&burned.next_state, pos(3)).unwrap();
    // Zero circulation can retain positive progress after an earlier floor burn.
    assert_eq!(exit.next_state.accounting(), Accounting::new(nn(3), nn(0)));
    assert_eq!(exit.reserve, Bounds::exact(nn(102)));
    assert_eq!(
        engine.backing(&exit.next_state).unwrap(),
        Bounds::exact(nn(0))
    );
}

struct ObservedCharacteristic(Rc<RefCell<Vec<AffineTransform<N>>>>);

impl Characteristic<N> for ObservedCharacteristic {
    type Calculator = LinearCalculator<N>;

    fn calculator(&self, affine: AffineTransform<N>) -> Self::Calculator {
        self.0.borrow_mut().push(affine);
        linear().calculator(affine)
    }
}

#[test]
fn floor_operations_and_contraction_do_not_rebase_or_evaluate_the_characteristic() {
    let views = Rc::new(RefCell::new(Vec::new()));
    let characteristic = ObservedCharacteristic(Rc::clone(&views));
    let engine = CharacteristicEngine::new(characteristic, scalar());
    let calculator: &dyn AvmCalculator<N> = &engine;
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
    assert!(views.borrow().is_empty());
    for (state, backing) in [(burn.next_state, 102), (exercise.next_state, 134)] {
        assert_eq!(state.segmentation(), raised().segmentation());
        assert_eq!(calculator.price(&state).unwrap(), Bounds::exact(nn(49)));
        assert_eq!(
            calculator.backing(&state).unwrap(),
            Bounds::exact(nn(backing))
        );
    }
    views.borrow_mut().clear();

    let characteristic = ObservedCharacteristic(Rc::clone(&views));
    compile_schedule(&characteristic, scalar(), contracted).unwrap();
    let recorded = views.take();
    assert_eq!(recorded.len(), 2);
    let main = recorded.iter().find(|a| a.y_scale() == nn(1)).unwrap();
    assert_eq!(main.anchor().x(), nn(4));
    assert_eq!(main.anchor().y(), Bounds::exact(fraction(57, 2)));
    assert!(
        recorded
            .iter()
            .all(|a| a.x_scale() == Pos::one() && a.x_translation() == N::ZERO)
    );

    let collapsed = contracted.contract_to(nn(2)).unwrap();
    compile_schedule(&characteristic, scalar(), collapsed).unwrap();
    let recorded = views.take();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].anchor().x(), nn(2));
    assert_eq!(recorded[0].anchor().y(), Bounds::exact(nn(16)));
    assert_eq!(recorded[0].y_scale(), nn(1));
}

#[test]
fn floor_raise_checks_all_circulating_backing_and_explicit_residual_limits() {
    let engine = engine();
    let issued = engine
        .quote_issue(&MarketState::launch(pos(1)), pos(6))
        .unwrap()
        .next_state;
    let with_options = engine
        .quote_issue_at_floor(&issued, pos(2))
        .unwrap()
        .next_state;
    // The net-issued area still matches, but the extra two tokens now need 30 more reserve.
    assert_eq!(
        engine
            .verify_floor_raise(&with_options, proposal(), FloorRaisePolicy::exact())
            .err(),
        Some(Error::BackingIncreased)
    );

    let surplus = FloorRaiseProposal {
        ramp_end: fraction(185, 32),
        ramp_width: fraction(5, 2),
        ..proposal()
    };
    assert_eq!(
        engine
            .verify_floor_raise(&issued, surplus, FloorRaisePolicy::exact())
            .err(),
        Some(Error::ReserveSurplusExceeded)
    );
    let accepted = engine
        .verify_floor_raise(&issued, surplus, FloorRaisePolicy::new(nn(0), nn(8)))
        .unwrap();
    assert_eq!(accepted.main_change, Bounds::exact(N::ZERO));
    assert_eq!(
        accepted.backing_change,
        Bounds::exact(fraction(2025, 256).get().checked_neg().unwrap())
    );

    let lowered_main = FloorRaiseProposal {
        floor: Pos::new(fraction(255, 16).get()).unwrap(),
        ..proposal()
    };
    assert_eq!(
        engine
            .verify_floor_raise(
                &issued,
                lowered_main,
                FloorRaisePolicy::new(fraction(1, 32), nn(1))
            )
            .err(),
        Some(Error::MainCurveChanged)
    );
    let accepted = engine
        .verify_floor_raise(
            &issued,
            lowered_main,
            FloorRaisePolicy::new(fraction(1, 16), fraction(3, 8)),
        )
        .unwrap();
    assert_eq!(
        accepted.backing_change,
        Bounds::exact(fraction(3, 8).get().checked_neg().unwrap())
    );

    // The candidate endpoint encloses the old price but also permits an increase.
    // Merely overlapping intervals is not enough to satisfy a zero allowance.
    let uncertain = FloorRaiseProposal {
        floor: Pos::new(nn(16).get().checked_sub(N::from_scaled_i128(12)).unwrap()).unwrap(),
        ramp_width: NonNeg::new(nn(2).get().checked_add(N::QUANTUM).unwrap()).unwrap(),
        ..proposal()
    };
    let candidate = compile_schedule(
        &linear(),
        scalar(),
        Segmentation::new(uncertain.floor, uncertain.ramp_end, uncertain.ramp_width).unwrap(),
    )
    .unwrap();
    let endpoint = candidate.y(nn(5)).unwrap();
    assert!(endpoint.contains(nn(41)) && !endpoint.is_exact());
    assert_eq!(
        engine
            .verify_floor_raise(&issued, uncertain, FloorRaisePolicy::exact())
            .err(),
        Some(Error::MainCurveChanged)
    );
}

#[test]
fn invalid_market_geometry_and_unrepresentable_quotes_return_errors() {
    assert_eq!(
        Segmentation::new(pos(1), nn(2), nn(3)),
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
    assert_eq!(
        engine.quote_redeem(&raised(), pos(7)).err(),
        Some(Error::InsufficientCirculation)
    );
    let max = NonNeg::new(N::MAX).unwrap();
    let at_limit =
        MarketState::new(Segmentation::launch(pos(1)), Accounting::new(max, max)).unwrap();
    assert_eq!(
        engine.quote_issue_at_floor(&at_limit, pos(1)).err(),
        Some(Error::ArithmeticFailure)
    );
    assert_eq!(
        engine.quote_issue(&at_limit, pos(1)).err(),
        Some(Error::ArithmeticFailure)
    );
}

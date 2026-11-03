use std::cell::RefCell;

use mayf_characteristic::{AffineTransform, Characteristic};
use mayf_characteristic_linear_driver::{LinearCalculator, LinearCharacteristic};
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{AboveOne, Bounds, DirectedRounding, FixedNumber, NonNeg, Pos};
use mayf_schedule_segmented::{
    MainReference, ScheduleError, Segmentation, ShiftedScheduleCalculator,
    compile_segmented_schedule,
};

type Fx = I64F64;

fn number(value: i64) -> Fx {
    Fx::checked_from_i128(value.into()).unwrap()
}

fn nn(value: i64) -> NonNeg<Fx> {
    NonNeg::new(number(value)).unwrap()
}

fn fraction(numerator: i128, denominator: u128) -> NonNeg<Fx> {
    NonNeg::new(
        Fx::checked_from_ratio_round(
            numerator,
            denominator.try_into().unwrap(),
            DirectedRounding::Floor,
        )
        .unwrap(),
    )
    .unwrap()
}

fn scalar() -> AboveOne<Fx> {
    AboveOne::new(fraction(25, 16).get()).unwrap()
}

fn linear() -> LinearCharacteristic<Fx> {
    LinearCharacteristic::new(Pos::from_i32(8).unwrap())
}

fn raised() -> Segmentation<Fx> {
    Segmentation::new(
        nn(16),
        nn(5),
        nn(2),
        scalar(),
        MainReference::exact(nn(0), nn(1)),
    )
    .unwrap()
}

// Recording the neutral input mapping catches rebasing errors invisible for a
// line: translating a linear characteristic cancels inside its affine gap.
#[derive(Default)]
struct ObservedCharacteristic {
    views: RefCell<Vec<AffineTransform<Fx>>>,
}

impl Characteristic<Fx> for ObservedCharacteristic {
    type Calculator = LinearCalculator<Fx>;

    fn calculator(&self, affine: AffineTransform<Fx>) -> Self::Calculator {
        self.views.borrow_mut().push(affine);
        linear().calculator(affine)
    }
}

#[test]
fn reshaping_floor_and_ramp_preserves_the_independent_main_view() {
    let characteristic = ObservedCharacteristic::default();
    let launch =
        compile_segmented_schedule(&characteristic, Segmentation::launch(nn(1), scalar())).unwrap();
    let initial_views = characteristic.views.take();
    let raised = compile_segmented_schedule(&characteristic, raised()).unwrap();
    let raised_views = characteristic.views.take();
    let main = initial_views[0];
    assert_eq!(initial_views.len(), 1);
    assert!(raised_views.contains(&main));
    assert!(
        raised_views
            .iter()
            .all(|view| { view.x_scale() == Pos::one() && view.x_translation() == Fx::ZERO })
    );
    assert_eq!(main.anchor().x(), nn(0));
    assert_eq!(main.anchor().y(), Bounds::exact(nn(1)));
    assert_eq!(main.y_scale(), nn(1));

    for calculator in [&launch, &raised] {
        assert_eq!(calculator.y(nn(6)).unwrap(), Bounds::exact(nn(49)));
        assert_eq!(
            calculator
                .integrate(nn(0), Pos::from_i32(6).unwrap())
                .unwrap(),
            Bounds::exact(nn(150))
        );
    }

    // Even a discontinuous candidate must expose preserved main, rather than
    // silently reconstructing it from the candidate ramp's end rate.
    let candidate = Segmentation::new(
        nn(20),
        nn(5),
        nn(2),
        scalar(),
        raised.segmentation().main_reference(),
    )
    .unwrap();
    let candidate = compile_segmented_schedule(&characteristic, candidate).unwrap();
    assert_eq!(candidate.y(nn(6)).unwrap(), raised.y(nn(6)).unwrap());
    assert!(candidate.y(fraction(39, 8)).unwrap().lower() > candidate.y(nn(5)).unwrap().upper());
}

#[test]
fn contraction_retains_prefix_price_and_progress_then_uses_unscaled_main() {
    let characteristic = ObservedCharacteristic::default();
    let original = compile_segmented_schedule(&characteristic, raised()).unwrap();
    for (remaining, width, rate, prefix) in [
        (0, 0, nn(16), nn(0)),
        (2, 0, nn(16), nn(32)),
        (3, 0, nn(16), nn(48)),
        (4, 1, fraction(57, 2), fraction(281, 4)),
    ] {
        let remaining = nn(remaining);
        let state = original.contract_to(remaining).unwrap();
        assert_eq!(state.ramp_end(), remaining);
        assert_eq!(state.ramp_width(), nn(width));
        assert_eq!(
            state.main_reference(),
            MainReference::exact(remaining, rate)
        );
        characteristic.views.borrow_mut().clear();
        let contracted = compile_segmented_schedule(&characteristic, state).unwrap();
        let views = characteristic.views.borrow();
        let main = views.iter().find(|view| view.y_scale() == nn(1)).unwrap();
        assert_eq!(main.x_translation(), Fx::ZERO);
        assert_eq!(main.anchor().x(), remaining);
        assert_eq!(
            contracted.y(remaining).unwrap(),
            original.y(remaining).unwrap()
        );
        assert_eq!(
            contracted.y(remaining.checked_add(nn(1)).unwrap()).unwrap(),
            Bounds::exact(rate.checked_add(nn(8)).unwrap())
        );
        if remaining > nn(0) {
            let width = Pos::try_from(remaining).unwrap();
            assert_eq!(
                contracted.integrate(nn(0), width).unwrap(),
                Bounds::exact(prefix)
            );
            assert_eq!(
                contracted.integrate(nn(0), width).unwrap(),
                original.integrate(nn(0), width).unwrap()
            );
        }
        assert_eq!(contracted.contract_to(remaining).unwrap(), state);
    }
    for remaining in [nn(5), nn(6)] {
        assert_eq!(
            original.contract_to(remaining).unwrap(),
            original.segmentation()
        );
    }
}

#[test]
fn signed_translations_consume_and_extend_tail_without_changing_components() {
    let original = raised();
    let core = compile_segmented_schedule(&linear(), original).unwrap();
    let shifted = ShiftedScheduleCalculator::new(core, Fx::ZERO).unwrap();
    let shifted = shifted.translate(number(-3)).unwrap();
    assert_eq!((shifted.ramp_start(), shifted.ramp_end()), (nn(0), nn(2)));
    assert_eq!(shifted.inner().segmentation(), original);
    assert_eq!(shifted.y(nn(3)).unwrap(), Bounds::exact(nn(49)));
    assert_eq!(
        shifted.integrate(nn(0), Pos::from_i32(3).unwrap()).unwrap(),
        Bounds::exact(nn(102))
    );

    let shifted = shifted.translate(number(2)).unwrap();
    assert_eq!((shifted.ramp_start(), shifted.ramp_end()), (nn(2), nn(4)));
    assert_eq!(shifted.horizontal_shift(), number(-1));
    assert_eq!(shifted.y(nn(5)).unwrap(), Bounds::exact(nn(49)));
    assert_eq!(
        shifted.integrate(nn(0), Pos::from_i32(5).unwrap()).unwrap(),
        Bounds::exact(nn(134))
    );
    assert_eq!(shifted.inner().segmentation(), original);
    assert_eq!(
        shifted.translate(number(-3)).err(),
        Some(ScheduleError::NegativeRampStart)
    );
}

#[test]
fn right_shifted_tail_never_evaluates_negative_characteristic_inputs() {
    let core = compile_segmented_schedule(&linear(), raised()).unwrap();
    let shifted = ShiftedScheduleCalculator::new(core, number(4)).unwrap();
    assert_eq!(shifted.y(nn(0)).unwrap(), Bounds::exact(nn(16)));
    assert_eq!(
        shifted.integrate(nn(0), Pos::from_i32(2).unwrap()).unwrap(),
        Bounds::exact(nn(32))
    );
    assert_eq!(
        shifted.integrate(nn(2), Pos::from_i32(5).unwrap()).unwrap(),
        Bounds::exact(nn(80))
    );
    assert_eq!(
        shifted
            .integrate(nn(0), Pos::from_i32(10).unwrap())
            .unwrap(),
        Bounds::exact(nn(214))
    );
    assert_eq!(shifted.y(nn(10)).unwrap(), Bounds::exact(nn(49)));
    assert_eq!(shifted.into_inner().segmentation(), raised());
}

#[test]
fn collapsed_ramp_does_not_compile_an_unused_overflowing_gain() {
    let characteristic = LinearCharacteristic::try_new(Fx::MAX).unwrap();
    let state = Segmentation::launch(nn(0), AboveOne::new(number(2)).unwrap());
    let calculator = compile_segmented_schedule(&characteristic, state).unwrap();
    assert_eq!(
        calculator.y(nn(1)).unwrap(),
        Bounds::exact(NonNeg::new(Fx::MAX).unwrap())
    );
    let area = calculator.integrate(nn(0), Pos::one()).unwrap();
    assert_eq!(area.lower().get().to_scaled_i128(), i128::MAX / 2);
    assert_eq!(area.upper().get().to_scaled_i128(), i128::MAX / 2 + 1);
}

#[test]
fn invalid_geometry_and_unrepresentable_arithmetic_return_errors() {
    let reference = MainReference::exact(nn(0), nn(1));
    assert_eq!(
        Segmentation::new(nn(1), nn(2), nn(3), scalar(), reference),
        Err(ScheduleError::RampWidthExceedsEnd)
    );
    assert_eq!(
        Segmentation::new(
            nn(1),
            nn(2),
            nn(0),
            scalar(),
            MainReference::exact(nn(3), nn(1))
        ),
        Err(ScheduleError::MainReferenceAfterRampEnd)
    );
    assert_eq!(
        Segmentation::new(nn(2), nn(0), nn(0), scalar(), reference),
        Err(ScheduleError::InvalidOriginReference)
    );
    let core = || compile_segmented_schedule(&linear(), raised()).unwrap();
    assert_eq!(
        ShiftedScheduleCalculator::new(core(), number(-4)).err(),
        Some(ScheduleError::NegativeRampStart)
    );
    assert_eq!(
        ShiftedScheduleCalculator::new(core(), Fx::MAX).err(),
        Some(ScheduleError::ArithmeticFailure)
    );
    let max = NonNeg::new(Fx::MAX).unwrap();
    let quantum = Pos::new(Fx::QUANTUM).unwrap();
    assert_eq!(
        core().integrate(max, quantum),
        Err(ScheduleError::ArithmeticFailure)
    );
    assert_eq!(core().y(max), Err(ScheduleError::ArithmeticFailure));
    let shifted = ShiftedScheduleCalculator::new(core(), number(-3)).unwrap();
    assert_eq!(shifted.y(max), Err(ScheduleError::ArithmeticFailure));
    assert_eq!(
        shifted.integrate(max, quantum),
        Err(ScheduleError::ArithmeticFailure)
    );
}

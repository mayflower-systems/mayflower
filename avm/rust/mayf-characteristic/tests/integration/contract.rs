use mayf_characteristic::{AffineTransform, AffineTransformError, Characteristic};
use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_number::{Bounds, FixedNumber, NonNeg};
use mayf_monotone::{MonotoneCalculator, MonotoneCalculatorError, MonotoneCalculatorResult};

type Fx = DashuFixed12;

fn fixed(value: i128) -> Fx {
    Fx::checked_from_i128(value).unwrap()
}

fn nonnegative(value: i128) -> NonNeg<Fx> {
    NonNeg::new(fixed(value)).unwrap()
}

#[derive(Clone, Copy)]
struct NeutralCharacteristic {
    slope: NonNeg<Fx>,
}

struct TestCalculator {
    slope: NonNeg<Fx>,
    affine: AffineTransform<Fx>,
}

impl Characteristic<Fx> for NeutralCharacteristic {
    type Calculator = TestCalculator;

    fn calculator(&self, affine: AffineTransform<Fx>) -> Self::Calculator {
        TestCalculator {
            slope: self.slope,
            affine,
        }
    }
}

impl MonotoneCalculator<Fx> for TestCalculator {
    fn y(&self, _x: NonNeg<Fx>) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        Err(MonotoneCalculatorError::InputOutOfDomain)
    }

    fn integrate(
        &self,
        _lo: NonNeg<Fx>,
        width: NonNeg<Fx>,
    ) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        if width == NonNeg::zero() {
            return Ok(Bounds::exact(NonNeg::zero()));
        }
        Err(MonotoneCalculatorError::InputOutOfDomain)
    }
}

#[test]
fn affine_validation_uses_transform_level_errors() {
    assert_eq!(
        AffineTransform::try_new(
            fixed(0),
            fixed(0),
            fixed(1),
            fixed(0),
            Bounds::exact(fixed(0)),
        ),
        Err(AffineTransformError::NonPositiveHorizontalScale)
    );
    assert_eq!(
        AffineTransform::try_new(
            fixed(1),
            fixed(0),
            fixed(-1),
            fixed(0),
            Bounds::exact(fixed(0)),
        ),
        Err(AffineTransformError::NegativeVerticalScale)
    );

    // A zero vertical scale is valid and represents a constant function.
    assert!(
        AffineTransform::try_new(
            fixed(1),
            fixed(0),
            fixed(0),
            fixed(3),
            Bounds::exact(fixed(4)),
        )
        .is_ok()
    );
}

#[test]
fn calculators_capture_affine_state_without_mutating_the_characteristic() {
    let characteristic = NeutralCharacteristic {
        slope: nonnegative(2),
    };
    let identity = characteristic.calculator(AffineTransform::identity());
    let lifted_transform = AffineTransform::try_new(
        fixed(1),
        fixed(0),
        fixed(3),
        fixed(2),
        Bounds::new(fixed(4), fixed(5)).unwrap(),
    )
    .unwrap();
    let lifted = characteristic.calculator(lifted_transform);

    assert_eq!(characteristic.slope, nonnegative(2));
    assert_eq!(identity.slope, lifted.slope);
    assert!(identity.affine.is_identity());
    assert_eq!(lifted.affine, lifted_transform);
}

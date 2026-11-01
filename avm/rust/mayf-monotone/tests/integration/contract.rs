use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_number::{Bounds, FixedNumber, NonNeg};
use mayf_monotone::{
    MonotoneCalculator, MonotoneCalculatorError, MonotoneCalculatorResult, require_nonnegative,
};

type Fx = DashuFixed12;

fn fixed(value: i128) -> Fx {
    Fx::checked_from_i128(value).unwrap()
}

fn nonnegative(value: i128) -> NonNeg<Fx> {
    NonNeg::new(fixed(value)).unwrap()
}

/// Small test adapter that deliberately computes in the signed driver before
/// proving that its output is non-negative.
struct ShiftedIdentity {
    shift: Fx,
}

impl MonotoneCalculator<Fx> for ShiftedIdentity {
    fn y(&self, x: NonNeg<Fx>) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        let raw = x
            .get()
            .checked_add(self.shift)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let output = NonNeg::new(raw).map_err(|_| MonotoneCalculatorError::NegativeOutput)?;
        Ok(Bounds::exact(output))
    }

    fn integrate(
        &self,
        _lo: NonNeg<Fx>,
        width: NonNeg<Fx>,
    ) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        if width == NonNeg::zero() {
            return Ok(Bounds::exact(NonNeg::zero()));
        }
        Err(MonotoneCalculatorError::ArithmeticFailure)
    }
}

#[test]
fn negative_output_is_a_semantic_calculator_error() {
    let calculator = ShiftedIdentity { shift: fixed(-1) };

    assert_eq!(
        calculator.y(nonnegative(0)),
        Err(MonotoneCalculatorError::NegativeOutput)
    );
}

#[test]
fn nonnegative_output_crosses_the_contract_as_a_refined_value() {
    let calculator = ShiftedIdentity { shift: fixed(-1) };
    let output = calculator.y(nonnegative(2)).unwrap();

    assert_eq!(output.lower(), nonnegative(1));
    assert_eq!(output.upper(), nonnegative(1));
}

#[test]
fn signed_enclosure_must_prove_the_output_is_nonnegative() {
    assert_eq!(
        require_nonnegative(Bounds::new(fixed(-1), fixed(1)).unwrap()),
        Err(MonotoneCalculatorError::NegativeOutput)
    );

    let result = require_nonnegative(Bounds::new(fixed(1), fixed(2)).unwrap()).unwrap();
    assert_eq!(result.lower(), nonnegative(1));
    assert_eq!(result.upper(), nonnegative(2));
}

#[test]
fn zero_width_and_zero_area_are_identity_operations() {
    let calculator = ShiftedIdentity { shift: fixed(0) };
    let zero = nonnegative(0);
    let two = nonnegative(2);
    let three = nonnegative(3);

    assert_eq!(calculator.integrate(two, zero), Ok(Bounds::exact(zero)));

    let upward = calculator
        .max_hi_for_area_at_or_below(two, Some(three), zero)
        .unwrap()
        .unwrap();
    assert_eq!(upward.endpoint, two);
    assert_eq!(upward.area, zero);

    let downward = calculator
        .max_lo_for_area_at_or_above(two, three, zero)
        .unwrap()
        .unwrap();
    assert_eq!(downward.endpoint, three);
    assert_eq!(downward.area, zero);
}

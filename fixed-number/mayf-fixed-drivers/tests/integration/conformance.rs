//! Backend conformance tests mirroring the shared numeric contract and the
//! `mayf-fixed-dashu` suite, with binary-grid expectations.

use core::num::NonZeroU128;

use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{
    BoundedExponential, DirectedRounding, FixedNumber, IntegralRounding, MathError, SignedFixed,
};

const RAW_ONE: i128 = 1i128 << 64;

fn from_int(value: i128) -> I64F64 {
    I64F64::checked_from_i128(value).unwrap()
}

#[test]
fn constants_define_the_binary_grid() {
    assert_eq!(I64F64::ZERO.to_scaled_i128(), 0);
    assert_eq!(I64F64::ONE.to_scaled_i128(), RAW_ONE);
    assert_eq!(I64F64::QUANTUM.to_scaled_i128(), 1);
    assert_eq!(I64F64::MAX.to_scaled_i128(), i128::MAX);
    assert_eq!(I64F64::MIN.to_scaled_i128(), i128::MIN);
    assert_eq!(RAW_ONE >> I64F64::FRAC_BITS, 1);
}

#[test]
fn from_ratio_uses_directed_quantum_rounding() {
    let denominator = NonZeroU128::new(3).unwrap();

    let lower = I64F64::checked_from_ratio_round(1, denominator, DirectedRounding::Floor).unwrap();
    let upper = I64F64::checked_from_ratio_round(1, denominator, DirectedRounding::Ceil).unwrap();

    assert_eq!(lower.to_scaled_i128(), 0x5555_5555_5555_5555);
    assert_eq!(upper.checked_sub(lower), Ok(I64F64::QUANTUM));
    assert!(lower.to_scaled_i128() * 3 < RAW_ONE);
    assert!(upper.to_scaled_i128() * 3 > RAW_ONE);
}

#[test]
fn negative_rounding_is_mathematical_floor_and_ceil() {
    let denominator = NonZeroU128::new(3).unwrap();

    let lower = I64F64::checked_from_ratio_round(-1, denominator, DirectedRounding::Floor).unwrap();
    let upper = I64F64::checked_from_ratio_round(-1, denominator, DirectedRounding::Ceil).unwrap();

    assert_eq!(lower.to_scaled_i128(), -0x5555_5555_5555_5556);
    assert_eq!(upper.to_scaled_i128(), -0x5555_5555_5555_5555);
    assert_eq!(upper.checked_sub(lower), Ok(I64F64::QUANTUM));
}

#[test]
fn multiplication_uses_directed_rounding() {
    let one_third = I64F64::ONE.checked_div_floor(from_int(3)).unwrap();

    let floor = one_third
        .checked_mul_round(one_third, DirectedRounding::Floor)
        .unwrap();
    let ceil = one_third
        .checked_mul_round(one_third, DirectedRounding::Ceil)
        .unwrap();

    assert_eq!(ceil.checked_sub(floor), Ok(I64F64::QUANTUM));
    let ninth_floor = I64F64::ONE.checked_div_floor(from_int(9)).unwrap();
    let ninth_ceil = I64F64::ONE.checked_div_ceil(from_int(9)).unwrap();
    assert!(floor <= ninth_floor);
    assert!(ninth_ceil <= ceil);
}

#[test]
fn fused_mul_div_uses_one_rounding_step() {
    let one = I64F64::ONE;
    let three = from_int(3);

    let divided = one.checked_div_floor(three).unwrap();
    let fused = one.checked_mul_div_floor(one, three).unwrap();

    assert_eq!(divided, fused);
}

#[test]
fn power_of_two_arithmetic_is_exact() {
    let eighth = I64F64::from_scaled_i128(RAW_ONE >> 3);
    let floor = eighth.checked_mul_floor(eighth).unwrap();
    let ceil = eighth.checked_mul_ceil(eighth).unwrap();

    assert_eq!(
        floor, ceil,
        "power-of-two products land on the grid exactly"
    );
    assert_eq!(floor.to_scaled_i128(), RAW_ONE >> 6);
}

#[test]
fn reports_checked_arithmetic_failures() {
    assert_eq!(
        I64F64::ONE.checked_div_floor(I64F64::ZERO),
        Err(MathError::DivisionByZero)
    );
    assert_eq!(
        I64F64::MAX.checked_add(I64F64::QUANTUM),
        Err(MathError::Overflow)
    );
    assert_eq!(I64F64::MIN.checked_neg(), Err(MathError::Overflow));
    assert_eq!(
        I64F64::checked_from_i128(i128::MAX),
        Err(MathError::Overflow)
    );
    assert_eq!(
        I64F64::checked_from_ratio_round(
            1,
            NonZeroU128::new(u128::MAX).unwrap(),
            DirectedRounding::Floor
        ),
        Ok(I64F64::ZERO)
    );
}

#[test]
fn exp_m1_bounds_handle_domain_edges() {
    let zero = I64F64::ZERO.checked_exp_m1_bounds().unwrap();
    assert_eq!(zero.lower(), I64F64::ZERO);
    assert_eq!(zero.upper(), I64F64::ZERO);

    assert_eq!(
        from_int(44).checked_exp_m1_bounds(),
        Err(MathError::Overflow)
    );

    let negative_one = I64F64::from_scaled_i128(-RAW_ONE);
    for saturated in [from_int(-45), from_int(-60)] {
        let bounds = saturated.checked_exp_m1_bounds().unwrap();
        assert_eq!(bounds.lower(), negative_one);
        assert_eq!(bounds.upper(), negative_one.checked_next_up().unwrap());
    }

    // Just inside the cutoff the full construction must agree with the
    // saturated enclosure it borders.
    let near_cutoff = from_int(-45).checked_next_up().unwrap();
    let bounds = near_cutoff.checked_exp_m1_bounds().unwrap();
    assert!(bounds.lower() >= negative_one);
    assert!(bounds.upper() <= from_int(-45).checked_exp_m1_bounds().unwrap().upper());
}

#[test]
fn exp_m1_bounds_are_ordered_and_tight_for_one() {
    let bounds = I64F64::ONE.checked_exp_m1_bounds().unwrap();

    // e - 1 = 1.71828182845904523536... ; raw = floor((e-1) * 2^64).
    assert!(bounds.lower() <= bounds.upper());
    let width = bounds
        .upper()
        .checked_sub(bounds.lower())
        .unwrap()
        .to_scaled_i128();
    assert!(width <= 64, "enclosure of expm1(1) is unexpectedly loose");

    // floor((e - 1) * 2^64) = 31696705136089705066.7478...
    let mid = I64F64::from_scaled_i128(31_696_705_136_089_705_066);
    assert!(bounds.lower() <= mid && mid <= bounds.upper());
}

#[test]
fn floor_to_u64_checks_negative_values_before_discarding_fractions() {
    for (raw, expected) in [
        (0, Ok(0)),
        (1, Ok(0)),
        (RAW_ONE - 1, Ok(0)),
        (RAW_ONE, Ok(1)),
        (RAW_ONE + 1, Ok(1)),
        (i128::MAX, Ok(i64::MAX as u64)),
        (-1, Err(MathError::NegativeToUnsigned)),
        (-RAW_ONE - 1, Err(MathError::NegativeToUnsigned)),
        (i128::MIN, Err(MathError::NegativeToUnsigned)),
    ] {
        assert_eq!(
            I64F64::from_scaled_i128(raw).checked_floor_to_u64(),
            expected,
            "raw={raw}"
        );
    }
}

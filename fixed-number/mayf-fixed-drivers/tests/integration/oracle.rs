//! Differential tests against dashu high-precision arithmetic.
//!
//! The oracle evaluates every quantity twice — once rounded toward negative
//! infinity, once toward positive infinity, at 256-bit precision — so its two
//! results bracket the mathematical value to within `2^-129` raw units. A
//! backend enclosure passes only if it contains the oracle bracket.

use dashu::base::{DivRem, Sign, Signed};
use dashu::float::{Context, FBig, round::Round, round::mode};
use dashu::integer::IBig;

use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{BoundedExponential, Bounds, DirectedRounding, FixedNumber, MathError};

const RAW_ONE: i128 = 1i128 << 64;
const LN2_LO_RAW: i128 = 12_786_308_645_202_655_659;
const ORACLE_PRECISION: usize = 256;

fn fbin_from_raw<R: Round>(raw: i128) -> FBig<R, 2> {
    FBig::from_parts(IBig::from(raw), -64)
}

/// Quantize a binary oracle float onto the `2^-64` grid with the float's own
/// rounding mode (`Down` floors, `Up` ceils).
fn raw_from_fbin<R: Round>(value: FBig<R, 2>) -> i128 {
    let scaled = value * FBig::<R, 2>::from_parts(IBig::ONE, 64);
    i128::try_from(scaled.to_int().value()).expect("oracle value fits i128")
}

/// High-precision bracket of `expm1(x)` on the raw grid.
fn expm1_oracle(raw: i128) -> (i128, i128) {
    let down = Context::<mode::Down>::new(ORACLE_PRECISION)
        .exp_m1(fbin_from_raw::<mode::Down>(raw).repr())
        .value();
    let up = Context::<mode::Up>::new(ORACLE_PRECISION)
        .exp_m1(fbin_from_raw::<mode::Up>(raw).repr())
        .value();

    (raw_from_fbin(down), raw_from_fbin(up))
}

/// Independent integer implementation of `(a * b) / d` with directed
/// rounding, mirroring the semantics contract of `FixedNumber`.
fn muldiv_oracle(
    left: i128,
    right: i128,
    divisor: i128,
    rounding: DirectedRounding,
) -> Result<i128, MathError> {
    if divisor == 0 {
        return Err(MathError::DivisionByZero);
    }

    let mut numerator = IBig::from(left) * IBig::from(right);
    let mut denominator = IBig::from(divisor);
    if denominator.is_negative() {
        numerator = -numerator;
        denominator = -denominator;
    }

    let numerator_is_negative = numerator.is_negative();
    let numerator_is_positive = numerator.sign() == Sign::Positive && !numerator.is_zero();
    let (mut quotient, remainder) = numerator.div_rem(denominator);
    if !remainder.is_zero() {
        match rounding {
            DirectedRounding::Floor if numerator_is_negative => quotient -= IBig::ONE,
            DirectedRounding::Ceil if numerator_is_positive => quotient += IBig::ONE,
            _ => {}
        }
    }

    i128::try_from(quotient).map_err(|_| MathError::Overflow)
}

/// Deterministic pseudo-random generator (PCG-style multiplier).
fn next_state(state: &mut u128) -> u128 {
    *state = state
        .wrapping_mul(0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645)
        .wrapping_add(0x1405_7b7e_f767_814f);
    *state
}

/// Draw an `i128` with widely varying magnitude.
fn next_i128(state: &mut u128) -> i128 {
    let value = next_state(state) as i128;
    let shift = (next_state(state) >> 121) as u32 % 120;
    value >> shift
}

#[test]
fn ln2_constants_enclose_the_true_value() {
    let two = FBig::<mode::Down, 2>::from_parts(IBig::from(2), 0);
    let down = Context::<mode::Down>::new(ORACLE_PRECISION)
        .ln(two.repr())
        .value();
    let two_up = FBig::<mode::Up, 2>::from_parts(IBig::from(2), 0);
    let up = Context::<mode::Up>::new(ORACLE_PRECISION)
        .ln(two_up.repr())
        .value();

    assert_eq!(raw_from_fbin(down), LN2_LO_RAW);
    assert_eq!(raw_from_fbin(up), LN2_LO_RAW + 1);
}

#[test]
fn muldiv_agrees_with_bigint_oracle() {
    let specials = [
        0i128,
        1,
        -1,
        2,
        -2,
        RAW_ONE,
        -RAW_ONE,
        RAW_ONE - 1,
        RAW_ONE + 1,
        i128::MAX,
        i128::MIN,
        i128::MIN + 1,
        i128::MAX / 3,
        1i128 << 100,
        -(1i128 << 100),
    ];

    let mut cases: Vec<(i128, i128, i128)> = Vec::new();
    for &a in &specials {
        for &b in &specials {
            for &d in &[1i128, -1, 3, -3, RAW_ONE, -RAW_ONE, i128::MAX, i128::MIN] {
                cases.push((a, b, d));
            }
        }
    }
    let mut state = 0x853c_49e6_748f_ea9b_da3e_39cb_94b9_5bdbu128;
    for _ in 0..4000 {
        cases.push((
            next_i128(&mut state),
            next_i128(&mut state),
            next_i128(&mut state),
        ));
    }

    for (a, b, d) in cases {
        if a >= 0 && b >= 0 && d >= 0 {
            let expected = muldiv_oracle(a, b, d, DirectedRounding::Floor)
                .and_then(|lo| muldiv_oracle(a, b, d, DirectedRounding::Ceil).map(|hi| (lo, hi)));
            let actual = (I64F64::from_scaled_i128(a).bounded() * I64F64::from_scaled_i128(b)
                / I64F64::from_scaled_i128(d))
            .bounds()
            .map(|bounds| bounds.into_raw().into_edges())
            .map(|(lo, hi)| (lo.to_scaled_i128(), hi.to_scaled_i128()));
            assert_eq!(actual, expected, "bounded {a} * {b} / {d}");
        }
        for rounding in [DirectedRounding::Floor, DirectedRounding::Ceil] {
            let mine = I64F64::from_scaled_i128(a)
                .checked_mul_div_round(
                    I64F64::from_scaled_i128(b),
                    I64F64::from_scaled_i128(d),
                    rounding,
                )
                .map(I64F64::to_scaled_i128);
            let expected = muldiv_oracle(a, b, d, rounding);

            assert_eq!(mine, expected, "{a} * {b} / {d} with {rounding:?}");
        }
    }
}

#[test]
fn optimized_q64_integral_divisors_agree_with_bigint_oracle() {
    let values = [
        i128::MIN,
        i128::MIN + 1,
        -(1i128 << 100),
        -RAW_ONE - 1,
        -RAW_ONE,
        -1,
        0,
        1,
        RAW_ONE,
        RAW_ONE + 1,
        1i128 << 100,
        i128::MAX - 1,
        i128::MAX,
    ];
    let divisors = [-67i128, -7, -3, -2, 2, 3, 7, 67];

    let mut cases = Vec::new();
    for &left in &values {
        for &right in &values {
            for &integral_divisor in &divisors {
                cases.push((left, right, integral_divisor));
            }
        }
    }

    let mut state = 0x4d59_5df4_d0f3_3173_1b03_734e_a44f_9e93u128;
    for _ in 0..4_000 {
        let magnitude = (next_state(&mut state) % 67 + 1) as i128;
        let integral_divisor = if next_state(&mut state) & 1 == 0 {
            magnitude
        } else {
            -magnitude
        };
        cases.push((
            next_i128(&mut state),
            next_i128(&mut state),
            integral_divisor,
        ));
    }

    for (left, right, integral_divisor) in cases {
        let scaled_divisor = integral_divisor * RAW_ONE;
        for rounding in [DirectedRounding::Floor, DirectedRounding::Ceil] {
            let mine = I64F64::from_scaled_i128(left)
                .checked_mul_div_round(
                    I64F64::from_scaled_i128(right),
                    I64F64::from_scaled_i128(scaled_divisor),
                    rounding,
                )
                .map(I64F64::to_scaled_i128);
            let expected = muldiv_oracle(left, right, scaled_divisor, rounding);

            assert_eq!(
                mine, expected,
                "{left} * {right} / ({integral_divisor} * 2^64) with {rounding:?}"
            );
        }
    }
}

fn assert_encloses_oracle(x_raw: i128) {
    let bounds: Bounds<I64F64> = I64F64::from_scaled_i128(x_raw)
        .checked_exp_m1_bounds()
        .unwrap_or_else(|error| panic!("expm1({x_raw}) failed: {error:?}"));
    let lower = bounds.lower().to_scaled_i128();
    let upper = bounds.upper().to_scaled_i128();
    let (oracle_lo, oracle_hi) = expm1_oracle(x_raw);

    assert!(lower <= upper, "inverted bounds at {x_raw}");
    assert!(
        lower <= oracle_lo,
        "lower bound above truth at {x_raw}: {lower} > {oracle_lo}"
    );
    assert!(
        oracle_hi <= upper,
        "upper bound below truth at {x_raw}: {upper} < {oracle_hi}"
    );
    assert!(lower >= -RAW_ONE, "expm1 must stay above -1 at {x_raw}");

    let k = x_raw.div_euclid(LN2_LO_RAW).max(0) as u32;
    let width_cap = 300i128 << k;
    assert!(
        upper - lower <= width_cap,
        "enclosure at {x_raw} unexpectedly loose: width {} > {width_cap}",
        upper - lower
    );
}

#[test]
fn expm1_bounds_enclose_oracle_at_curated_points() {
    let half = RAW_ONE / 2;
    let curated = [
        1,
        -1,
        2,
        -2,
        RAW_ONE >> 32,
        -(RAW_ONE >> 32),
        half,
        -half,
        RAW_ONE,
        -RAW_ONE,
        2 * RAW_ONE,
        -3 * RAW_ONE,
        // The decimal-backend counterexample input from VERIFICATION_PLAN.
        5 * RAW_ONE,
        20 * RAW_ONE,
        -20 * RAW_ONE,
        43 * RAW_ONE,
        43 * RAW_ONE + half,
        -44 * RAW_ONE - half,
        -45 * RAW_ONE + 1,
    ];

    for x in curated {
        assert_encloses_oracle(x);
    }
}

#[test]
fn expm1_bounds_enclose_oracle_at_reduction_boundaries() {
    for multiple in [-40i128, -3, -1, 1, 3, 10, 32, 62] {
        for offset in -2i128..=2 {
            assert_encloses_oracle(multiple * LN2_LO_RAW + offset);
        }
    }
}

#[test]
fn expm1_bounds_enclose_oracle_across_the_domain() {
    let min = -45 * RAW_ONE + 1;
    let max = 43 * RAW_ONE + RAW_ONE / 2;
    let span = (max - min) as u128;

    let mut state = 0x9e37_79b9_7f4a_7c15_f39c_c060_5ced_c834u128;
    for _ in 0..400 {
        let x = min + (next_state(&mut state) % span) as i128;
        if x != 0 {
            assert_encloses_oracle(x);
        }
    }
}

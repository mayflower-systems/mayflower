//! Differential checks for both unsigned fixed-point storage flavors.

use dashu::base::DivRem;
use dashu::integer::UBig;

use crypto_bigint::{Encoding, U256};
use mayf_fixed_drivers::{U64F64, U128F128};
use mayf_fixed_number::{DirectedRounding, FixedNumber, MathError};

fn div_round_oracle(
    numerator: UBig,
    denominator: UBig,
    maximum: &UBig,
    rounding: DirectedRounding,
) -> Result<UBig, MathError> {
    if denominator == UBig::ZERO {
        return Err(MathError::DivisionByZero);
    }
    let (mut quotient, remainder) = numerator.div_rem(denominator);
    if remainder != UBig::ZERO && rounding == DirectedRounding::Ceil {
        quotient += UBig::ONE;
    }
    if &quotient > maximum {
        Err(MathError::Overflow)
    } else {
        Ok(quotient)
    }
}

fn next_state(state: &mut u128) -> u128 {
    *state = state
        .wrapping_mul(0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645)
        .wrapping_add(0x1405_7b7e_f767_814f);
    *state
}

#[test]
fn u64f64_operations_agree_with_bigint_oracle() {
    let scale = UBig::ONE << 64;
    let maximum = (UBig::ONE << 128) - UBig::ONE;
    let values = [
        0,
        1,
        2,
        (1u128 << 64) - 1,
        1u128 << 64,
        (1u128 << 64) + 1,
        1u128 << 100,
        u128::MAX - 1,
        u128::MAX,
    ];

    let mut cases = Vec::new();
    for left in values {
        for right in values {
            for divisor in values {
                cases.push((left, right, divisor));
            }
        }
    }
    let mut state = 0x853c_49e6_748f_ea9b_da3e_39cb_94b9_5bdbu128;
    for _ in 0..1_000 {
        cases.push((
            next_state(&mut state),
            next_state(&mut state),
            next_state(&mut state),
        ));
    }

    for (left, right, divisor) in cases {
        let left_fixed = U64F64::from_scaled_u128(left);
        let right_fixed = U64F64::from_scaled_u128(right);
        let divisor_fixed = U64F64::from_scaled_u128(divisor);
        let expected = div_round_oracle(
            UBig::from(left) * UBig::from(right),
            UBig::from(divisor),
            &maximum,
            DirectedRounding::Floor,
        )
        .and_then(|lo| {
            div_round_oracle(
                UBig::from(left) * UBig::from(right),
                UBig::from(divisor),
                &maximum,
                DirectedRounding::Ceil,
            )
            .map(|hi| (lo, hi))
        });
        let actual = (left_fixed.bounded() * right_fixed / divisor_fixed)
            .bounds()
            .map(|bounds| bounds.into_raw().into_edges())
            .map(|(lo, hi)| {
                (
                    UBig::from(lo.to_scaled_u128()),
                    UBig::from(hi.to_scaled_u128()),
                )
            });
        assert_eq!(actual, expected, "bounded {left} * {right} / {divisor}");
        for rounding in [DirectedRounding::Floor, DirectedRounding::Ceil] {
            let actual_mul = left_fixed
                .checked_mul_round(right_fixed, rounding)
                .map(U64F64::to_scaled_u128)
                .map(UBig::from);
            let expected_mul = div_round_oracle(
                UBig::from(left) * UBig::from(right),
                scale.clone(),
                &maximum,
                rounding,
            );
            assert_eq!(actual_mul, expected_mul, "multiply {left} * {right}");

            let actual_div = left_fixed
                .checked_div_round(right_fixed, rounding)
                .map(U64F64::to_scaled_u128)
                .map(UBig::from);
            let expected_div = div_round_oracle(
                UBig::from(left) * &scale,
                UBig::from(right),
                &maximum,
                rounding,
            );
            assert_eq!(actual_div, expected_div, "divide {left} / {right}");

            let actual_mul_div = left_fixed
                .checked_mul_div_round(right_fixed, divisor_fixed, rounding)
                .map(U64F64::to_scaled_u128)
                .map(UBig::from);
            let expected_mul_div = div_round_oracle(
                UBig::from(left) * UBig::from(right),
                UBig::from(divisor),
                &maximum,
                rounding,
            );
            assert_eq!(
                actual_mul_div, expected_mul_div,
                "multiply-divide {left} * {right} / {divisor}"
            );
        }
    }
}

fn u256_from_halves(high: u128, low: u128) -> U256 {
    U256::from_u128(low).bitor(&U256::from_u128(high).shl_vartime(128))
}

fn ubig_from_u256(value: U256) -> UBig {
    UBig::from_le_bytes(&value.to_le_bytes())
}

#[test]
fn u128f128_operations_agree_with_bigint_oracle() {
    // The wide grid deliberately has no full-width multiply or divide. What it
    // does have is a multiplication by the narrow grid, an exact raw ratio in
    // `u64` terms, and a narrowing conversion.
    let narrow_scale = UBig::ONE << 64;
    let narrow_maximum = (UBig::ONE << 128) - UBig::ONE;
    let wide_maximum = (UBig::ONE << 256) - UBig::ONE;
    let wide_values = [
        U256::ZERO,
        U256::ONE,
        U256::from_u128(2),
        U256::ONE.shl_vartime(128),
        U256::ONE.shl_vartime(128).wrapping_add(&U256::ONE),
        U256::ONE.shl_vartime(200),
        U256::MAX,
    ];
    let narrow_values = [
        0u128,
        1,
        2,
        (1u128 << 64) - 1,
        1u128 << 64,
        1u128 << 100,
        u128::MAX,
    ];
    let ratios = [
        (0u64, 1u64),
        (1, 1),
        (1, 3),
        (3, 2),
        (u64::MAX - 1, u64::MAX),
        (u64::MAX, u64::MAX),
        (u64::MAX, u64::MAX - 1),
    ];

    let mut state = 0xda3e_39cb_94b9_5bdb_853c_49e6_748f_ea9bu128;
    let mut cases: Vec<(U256, u128)> = Vec::new();
    for wide in wide_values {
        for narrow in narrow_values {
            cases.push((wide, narrow));
        }
    }
    for _ in 0..500 {
        cases.push((
            u256_from_halves(next_state(&mut state), next_state(&mut state)),
            next_state(&mut state),
        ));
    }

    for (wide, narrow) in cases {
        let wide_fixed = U128F128::from_scaled_u256(wide);
        let wide_oracle = ubig_from_u256(wide);

        for rounding in [DirectedRounding::Floor, DirectedRounding::Ceil] {
            let actual = wide_fixed
                .checked_mul_u64f64_round(U64F64::from_scaled_u128(narrow), rounding)
                .map(U128F128::to_scaled_u256)
                .map(ubig_from_u256);
            let expected = div_round_oracle(
                &wide_oracle * UBig::from(narrow),
                narrow_scale.clone(),
                &wide_maximum,
                rounding,
            );
            assert_eq!(actual, expected, "wide by narrow multiply");

            let actual = wide_fixed
                .checked_to_u64f64_round(rounding)
                .map(U64F64::to_scaled_u128)
                .map(UBig::from);
            let expected = div_round_oracle(
                wide_oracle.clone(),
                narrow_scale.clone(),
                &narrow_maximum,
                rounding,
            );
            assert_eq!(actual, expected, "narrowing conversion");

            for (multiplier, divisor) in ratios {
                let actual = wide_fixed
                    .checked_mul_div_u64_round(
                        multiplier,
                        core::num::NonZeroU64::new(divisor).unwrap(),
                        rounding,
                    )
                    .map(U128F128::to_scaled_u256)
                    .map(ubig_from_u256);
                let expected = div_round_oracle(
                    &wide_oracle * UBig::from(multiplier),
                    UBig::from(divisor),
                    &wide_maximum,
                    rounding,
                );
                assert_eq!(actual, expected, "raw ratio {multiplier}/{divisor}");
            }
        }
    }
}

#[test]
fn optimized_u128f128_ratios_agree_with_bigint_oracle() {
    let scale = UBig::ONE << 128;
    let maximum = (UBig::ONE << 256) - UBig::ONE;
    let numerators = [0, 1, 2, i64::MAX as i128, i128::MAX];
    let denominators = [
        1,
        2,
        3,
        u128::from(u64::MAX),
        u128::from(u64::MAX) + 1,
        u128::MAX,
    ];

    for numerator in numerators {
        for denominator in denominators {
            let denominator_nonzero = core::num::NonZeroU128::new(denominator).unwrap();
            for rounding in [DirectedRounding::Floor, DirectedRounding::Ceil] {
                let actual =
                    U128F128::checked_from_ratio_round(numerator, denominator_nonzero, rounding)
                        .map(U128F128::to_scaled_u256)
                        .map(ubig_from_u256);
                let expected = div_round_oracle(
                    UBig::from(numerator as u128) * &scale,
                    UBig::from(denominator),
                    &maximum,
                    rounding,
                );
                assert_eq!(actual, expected, "{numerator}/{denominator}");
            }
        }
    }
}

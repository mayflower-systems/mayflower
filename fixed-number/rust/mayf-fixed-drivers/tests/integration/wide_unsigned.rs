//! Independent arbitrary-precision oracle for the wide integral driver.
use core::num::NonZeroU128;
use crypto_bigint::U256;
use dashu::{base::DivRem, integer::UBig};
use mayf_fixed_drivers::{U64F64, U192F64};
use mayf_fixed_number::{DirectedRounding, FixedNumber, IntegralRounding, MathError};

fn raw(value: U192F64) -> UBig {
    UBig::from_le_bytes(&value.to_le_bytes())
}
fn fixed(value: &UBig) -> U192F64 {
    let mut bytes = [0_u8; 32];
    let input = value.to_le_bytes();
    bytes[..input.len()].copy_from_slice(&input);
    U192F64::from_le_bytes(bytes)
}
fn rounded(
    numerator: UBig,
    denominator: UBig,
    rounding: DirectedRounding,
) -> Result<UBig, MathError> {
    if denominator == UBig::ZERO {
        return Err(MathError::DivisionByZero);
    }
    let (mut quotient, remainder) = numerator.div_rem(denominator);
    if remainder != UBig::ZERO && rounding == DirectedRounding::Ceil {
        quotient += UBig::ONE;
    }
    if quotient >= (UBig::ONE << 256) {
        Err(MathError::Overflow)
    } else {
        Ok(quotient)
    }
}

fn check(left: U192F64, right: U192F64, divisor: U192F64) {
    let l = raw(left);
    let r = raw(right);
    let d = raw(divisor);
    let scale = UBig::ONE << 64;
    let expected = rounded(&l * &r, d.clone(), DirectedRounding::Floor)
        .and_then(|lo| rounded(&l * &r, d.clone(), DirectedRounding::Ceil).map(|hi| (lo, hi)));
    let actual = (left.bounded() * right / divisor)
        .bounds()
        .map(|bounds| bounds.into_raw().into_edges())
        .map(|(lo, hi)| (raw(lo), raw(hi)));
    assert_eq!(actual, expected, "bounded {l} * {r} / {d}");
    for direction in [DirectedRounding::Floor, DirectedRounding::Ceil] {
        assert_eq!(
            left.checked_mul_round(right, direction).map(raw),
            rounded(&l * &r, scale.clone(), direction),
            "multiply {l} * {r} / scale {direction:?}"
        );
        assert_eq!(
            left.checked_div_round(right, direction).map(raw),
            rounded(&l * &scale, r.clone(), direction),
            "divide {l} * scale / {r} {direction:?}"
        );
        assert_eq!(
            left.checked_mul_div_round(right, divisor, direction)
                .map(raw),
            rounded(&l * &r, d.clone(), direction),
            "fused {l} * {r} / {d} {direction:?}"
        );
    }
}

#[test]
fn directed_operations_match_bigint_on_boundaries_and_wide_intermediates() {
    let mut values = vec![UBig::ZERO, UBig::ONE, UBig::from(2_u8)];
    for bit in [64, 128, 192, 255] {
        let center = UBig::ONE << bit;
        values.push(&center - UBig::ONE);
        values.push(center.clone());
        values.push(center + UBig::ONE);
    }
    let maximum = (UBig::ONE << 256) - UBig::ONE;
    values.extend([maximum.clone() - UBig::ONE, maximum]);
    for left in &values {
        for right in &values {
            for divisor in &values {
                check(fixed(left), fixed(right), fixed(divisor));
            }
        }
    }
}

fn next(state: &mut u128) -> u128 {
    *state = state
        .wrapping_mul(0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645)
        .wrapping_add(0x1405_7b7e_f767_814f);
    *state
}
fn random(state: &mut u128, shift: usize) -> U192F64 {
    let value = U256::from_u128(next(state))
        .shl_vartime(128)
        .bitor(&U256::from_u128(next(state)))
        .shr_vartime(shift);
    U192F64::from_scaled_u256(value)
}

#[test]
fn random_operations_cover_fast_and_general_division_paths() {
    let mut state = 0x853c_49e6_748f_ea9b_da3e_39cb_94b9_5bdb_u128;
    for index in 0..2000 {
        let left = random(&mut state, [0, 64, 128, 192][index % 4]);
        let right = random(&mut state, [0, 63, 128, 192][index % 4]);
        let divisor = random(&mut state, [0, 64, 128, 191][(index / 4) % 4]);
        check(left, right, divisor);
        check(left, U192F64::from_u64(next(&mut state) as u64), divisor);
    }
}

#[test]
fn ratio_conversion_storage_and_integral_rounding_are_exact() {
    for numerator in [0, 1, 3, i64::MAX as i128, i128::MAX] {
        for denominator in [1, 3, 1_u128 << 64, u128::MAX] {
            for direction in [DirectedRounding::Floor, DirectedRounding::Ceil] {
                let value = U192F64::checked_from_ratio_round(
                    numerator,
                    NonZeroU128::new(denominator).unwrap(),
                    direction,
                )
                .unwrap();
                assert_eq!(
                    Ok(raw(value)),
                    rounded(
                        UBig::from(numerator as u128) << 64,
                        UBig::from(denominator),
                        direction
                    )
                );
                assert_eq!(U192F64::from_le_bytes(value.to_le_bytes()), value);
            }
        }
        let integer = U192F64::checked_from_i128(numerator).unwrap();
        assert_eq!(
            integer.checked_floor_to_u64(),
            u64::try_from(numerator).map_err(|_| MathError::Overflow)
        );
    }
    assert_eq!(
        U192F64::checked_from_i128(-1),
        Err(MathError::NegativeToUnsigned)
    );
    assert_eq!(
        U192F64::checked_from_ratio_round(
            -1,
            NonZeroU128::new(1).unwrap(),
            DirectedRounding::Floor
        ),
        Err(MathError::NegativeToUnsigned)
    );
    assert_eq!(U192F64::QUANTUM.checked_floor_to_u64(), Ok(0));
    // Fractional slack at u64::MAX still fits, but any high integer limb
    // must be rejected rather than discarded during narrowing.
    assert_eq!(
        U192F64::from_scaled_u256(U256::from_u128(u128::MAX)).checked_floor_to_u64(),
        Ok(u64::MAX)
    );
    for raw in [
        U256::ONE.shl_vartime(128),
        U256::ONE.shl_vartime(255),
        U256::MAX,
    ] {
        assert_eq!(
            U192F64::from_scaled_u256(raw).checked_floor_to_u64(),
            Err(MathError::Overflow)
        );
    }
    for bits in [0, 1, 1_u128 << 64, u128::MAX] {
        let narrow = U64F64::from_scaled_u128(bits);
        let wide = U192F64::from(narrow);
        assert_eq!(raw(wide), UBig::from(bits));
        assert_eq!(U64F64::try_from(wide), Ok(narrow));
    }
    assert_eq!(U64F64::try_from(U192F64::MAX), Err(MathError::Overflow));
}

#[test]
fn full_u64_deposit_times_full_u64_lifetime_fits_without_precision_loss() {
    let maximum = U192F64::from_u64(u64::MAX);
    let product = maximum
        .checked_mul_round(maximum, DirectedRounding::Floor)
        .unwrap();
    assert_eq!(
        raw(product),
        (UBig::from(u64::MAX) * UBig::from(u64::MAX)) << 64
    );
    assert_eq!(
        product
            .checked_div_round(maximum, DirectedRounding::Ceil)
            .unwrap(),
        maximum
    );
    let split = product
        .checked_mul_div_round(
            U192F64::from(U64F64::from_scaled_u128(1_u128 << 63)),
            maximum,
            DirectedRounding::Floor,
        )
        .unwrap();
    assert_eq!(raw(split), UBig::from(u64::MAX) << 63);
}

#[test]
fn overflow_underflow_and_ceiling_at_the_storage_edge_are_checked() {
    assert_eq!(
        U192F64::MAX.checked_add(U192F64::QUANTUM),
        Err(MathError::Overflow)
    );
    assert_eq!(
        U192F64::ZERO.checked_sub(U192F64::QUANTUM),
        Err(MathError::Overflow)
    );

    let max = (UBig::ONE << 256) - UBig::ONE;
    let left = fixed(&(&max - UBig::ONE));
    let divisor = fixed(&(&max - UBig::from(2_u8)));
    assert_eq!(
        left.checked_mul_div_round(left, divisor, DirectedRounding::Floor),
        Ok(U192F64::MAX)
    );
    assert_eq!(
        left.checked_mul_div_round(left, divisor, DirectedRounding::Ceil),
        Err(MathError::Overflow)
    );
    assert_eq!(
        U192F64::ZERO.checked_mul_div_round(U192F64::ZERO, U192F64::ZERO, DirectedRounding::Floor),
        Err(MathError::DivisionByZero)
    );
}

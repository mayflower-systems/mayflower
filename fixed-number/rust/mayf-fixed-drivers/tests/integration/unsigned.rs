use core::mem::size_of;
use core::num::{NonZeroU64, NonZeroU128};

use crypto_bigint::U256;
use mayf_fixed_drivers::{I64F64, U64F64, U128F128};
use mayf_fixed_number::{DirectedRounding, FixedNumber, IntegralRounding, MathError};

#[test]
fn flavors_have_the_requested_storage_widths_and_grids() {
    assert_eq!(size_of::<U64F64>(), 16);
    assert_eq!(size_of::<U128F128>(), 32);

    assert_eq!(U64F64::FRAC_BITS, 64);
    assert_eq!(U64F64::ONE.to_scaled_u128(), 1u128 << 64);
    assert_eq!(U64F64::QUANTUM.to_scaled_u128(), 1);

    assert_eq!(U128F128::FRAC_BITS, 128);
    assert_eq!(U128F128::ONE.to_scaled_u256(), U256::ONE.shl_vartime(128));
    assert_eq!(U128F128::QUANTUM.to_scaled_u256(), U256::ONE);
}

#[test]
fn stable_bytes_round_trip_each_backing_width() {
    let narrow = U64F64::from_scaled_u128(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210);
    assert_eq!(U64F64::from_le_bytes(narrow.to_le_bytes()), narrow);

    let wide_raw =
        U256::from_be_hex("0123456789abcdeffedcba987654321000112233445566778899aabbccddeeff");
    let wide = U128F128::from_scaled_u256(wide_raw);
    assert_eq!(U128F128::from_le_bytes(wide.to_le_bytes()), wide);
}

#[test]
fn signed_q64_conversion_is_exact_and_rejects_negative_values() {
    let positive = I64F64::from_scaled_i128(0x1234_5678_9abc_def0);
    assert_eq!(
        U64F64::try_from(positive).unwrap().to_scaled_u128(),
        0x1234_5678_9abc_def0
    );
    assert_eq!(
        U128F128::try_from(positive).unwrap().to_scaled_u256(),
        U256::from_u128(0x1234_5678_9abc_def0).shl_vartime(64)
    );

    let negative = I64F64::from_scaled_i128(-1);
    assert_eq!(
        U64F64::try_from(negative),
        Err(MathError::NegativeToUnsigned)
    );
    assert_eq!(
        U128F128::try_from(negative),
        Err(MathError::NegativeToUnsigned)
    );
}

#[test]
fn directed_ratio_construction_separates_the_endpoints_on_both_grids() {
    let denominator = NonZeroU128::new(3).unwrap();

    let narrow_floor =
        U64F64::checked_from_ratio_round(1, denominator, DirectedRounding::Floor).unwrap();
    let narrow_ceil =
        U64F64::checked_from_ratio_round(1, denominator, DirectedRounding::Ceil).unwrap();
    assert_eq!(narrow_ceil.checked_sub(narrow_floor), Ok(U64F64::QUANTUM));

    let wide_floor =
        U128F128::checked_from_ratio_round(1, denominator, DirectedRounding::Floor).unwrap();
    let wide_ceil =
        U128F128::checked_from_ratio_round(1, denominator, DirectedRounding::Ceil).unwrap();
    assert_eq!(wide_ceil.checked_sub(wide_floor), Ok(U128F128::QUANTUM));

    assert_eq!(
        U64F64::checked_from_i128(-1),
        Err(MathError::NegativeToUnsigned)
    );
    assert_eq!(
        U128F128::checked_from_ratio_round(-1, denominator, DirectedRounding::Floor),
        Err(MathError::NegativeToUnsigned)
    );
}
#[test]
fn widening_multiply_and_directed_narrow_are_scale_correct() {
    let one_and_a_half = U64F64::from_scaled_u128((3u128 << 64) / 2);
    let two = U64F64::from_u64(2);
    let product = one_and_a_half.widening_mul(two);
    assert_eq!(product, U128F128::from_u64(3));

    let narrow_quantum_in_wide = U128F128::from_scaled_u256(U256::ONE.shl_vartime(63));
    assert_eq!(
        narrow_quantum_in_wide.checked_to_u64f64_floor(),
        Ok(U64F64::ZERO)
    );
    assert_eq!(
        narrow_quantum_in_wide.checked_to_u64f64_ceil(),
        Ok(U64F64::QUANTUM)
    );
}

#[test]
fn raw_u64_ratio_scaling_matches_general_fixed_arithmetic_at_boundaries() {
    // The wide grid has no general multiply-divide to compare against; its
    // `checked_mul_div_u64_round` is checked against arbitrary precision in
    // `tests/integration/unsigned_oracle.rs` instead.
    let narrow_values = [
        U64F64::ZERO,
        U64F64::QUANTUM,
        U64F64::ONE.checked_sub(U64F64::QUANTUM).unwrap(),
        U64F64::ONE,
        U64F64::from_scaled_u128(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210),
        U64F64::MAX,
    ];
    let ratios = [
        (0, 1),
        (1, 1),
        (1, 3),
        (3, 2),
        (u64::MAX - 1, u64::MAX),
        (u64::MAX, u64::MAX),
        (u64::MAX, u64::MAX - 1),
    ];

    for value in narrow_values {
        for (multiplier, divisor) in ratios {
            let divisor_nonzero = NonZeroU64::new(divisor).unwrap();
            for rounding in [DirectedRounding::Floor, DirectedRounding::Ceil] {
                let specialized =
                    value.checked_mul_div_u64_round(multiplier, divisor_nonzero, rounding);
                let general = value.checked_mul_div_round(
                    U64F64::from_u64(multiplier),
                    U64F64::from_u64(divisor),
                    rounding,
                );
                assert_eq!(
                    specialized, general,
                    "Q64.64 raw ratio mismatch for {value:?} * {multiplier} / {divisor}"
                );
            }
        }
    }
}

#[test]
fn overflow_is_checked_on_both_grids_and_the_narrow_one_floors() {
    assert_eq!(
        U64F64::MAX.checked_add(U64F64::QUANTUM),
        Err(MathError::Overflow)
    );
    assert_eq!(
        U128F128::MAX.checked_add(U128F128::QUANTUM),
        Err(MathError::Overflow)
    );

    let narrow = U64F64::from_scaled_u128((7u128 << 64) + 1);
    assert_eq!(narrow.checked_floor_to_u64(), Ok(7));
}

#[test]
fn floor_to_u64_accepts_fractional_values_at_the_unsigned_limit() {
    for whole in [0, 1, 7, u64::MAX] {
        for fraction in [0, 1, u64::MAX] {
            let value = U64F64::from_scaled_u128((u128::from(whole) << 64) | u128::from(fraction));
            assert_eq!(value.checked_floor_to_u64(), Ok(whole));
        }
    }
}

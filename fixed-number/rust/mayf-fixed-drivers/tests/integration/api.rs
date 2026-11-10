use core::mem::size_of;
use core::num::NonZeroU128;

use dashu::base::{DivRem, Sign, Signed};
use dashu::integer::IBig;
use mayf_fixed_drivers::{I64F64, U64F64, U128F128, U192F64};
use mayf_fixed_number::{
    DirectedRounding, FixedNumber, MathError, NonNeg, Pos, SignedFixed, UnsignedFixed,
};

#[test]
fn capabilities_and_refinements_share_one_representation() {
    fn subtract<N: SignedFixed>(amount: Pos<N>, adjustment: N) -> N {
        amount.get().checked_sub(adjustment).unwrap()
    }
    fn assert_unsigned<N: UnsignedFixed>() {}
    assert_unsigned::<U64F64>();
    assert_unsigned::<U192F64>();
    assert_eq!(NonNeg::from_unsigned(U64F64::MAX).get(), U64F64::MAX);
    assert_eq!(NonNeg::from_unsigned(U192F64::MAX).get(), U192F64::MAX);
    assert_eq!(NonNeg::from_unsigned(U64F64::ZERO), NonNeg::zero());

    let amount = Pos::new(I64F64::from(2_i64)).unwrap();
    assert_eq!(subtract(amount, I64F64::from(3_i64)), I64F64::from(-1_i64));
    assert_eq!(size_of::<Pos<I64F64>>(), size_of::<I64F64>());
    assert_eq!(size_of::<I64F64>(), 16);
    assert_eq!(size_of::<U64F64>(), 16);
    assert_eq!(size_of::<U192F64>(), 32);
    assert_eq!(size_of::<U128F128>(), 32);
}

#[test]
fn exact_conversions_check_sign_range_and_fraction_independently() {
    let signed_max = U64F64::from_scaled_u128(i128::MAX as u128);
    assert_eq!(I64F64::try_from(signed_max), Ok(I64F64::MAX));
    assert_eq!(U64F64::try_from(I64F64::MAX), Ok(signed_max));
    assert_eq!(
        I64F64::try_from(signed_max.checked_add(U64F64::QUANTUM).unwrap()),
        Err(MathError::Overflow)
    );
    assert_eq!(I64F64::try_from(U64F64::MAX), Err(MathError::Overflow));
    for negative in [I64F64::MIN, I64F64::from_scaled_i128(-1)] {
        assert_eq!(
            U64F64::try_from(negative),
            Err(MathError::NegativeToUnsigned)
        );
        assert_eq!(
            U192F64::try_from(negative),
            Err(MathError::NegativeToUnsigned)
        );
        assert_eq!(
            U128F128::try_from(negative),
            Err(MathError::NegativeToUnsigned)
        );
    }

    for value in [
        U64F64::ZERO,
        U64F64::QUANTUM,
        U64F64::ONE,
        signed_max,
        U64F64::MAX,
    ] {
        assert_eq!(U64F64::try_from(U192F64::from(value)), Ok(value));
        assert_eq!(U64F64::try_from(U128F128::from(value)), Ok(value));
    }
    let fraction = U128F128::ONE.checked_add(U128F128::QUANTUM).unwrap();
    assert_eq!(U64F64::try_from(fraction), Err(MathError::Inexact));
    assert_eq!(fraction.checked_to_u64f64_floor(), Ok(U64F64::ONE));
    assert_eq!(
        fraction.checked_to_u64f64_ceil(),
        U64F64::ONE.checked_add(U64F64::QUANTUM)
    );
    assert_eq!(U64F64::try_from(U128F128::MAX), Err(MathError::Overflow));
    assert_eq!(U64F64::try_from(U192F64::MAX), Err(MathError::Overflow));
}

#[test]
fn signed_storage_and_whole_conversions_preserve_extreme_values() {
    assert_eq!(I64F64::from(i64::MIN), I64F64::MIN);
    assert_eq!(
        I64F64::from(i64::MAX).to_scaled_i128(),
        i128::from(i64::MAX) << 64
    );
    assert_eq!(
        U64F64::from(u64::MAX).to_scaled_u128(),
        u128::from(u64::MAX) << 64
    );
    for raw in [i128::MIN, i128::MIN + 1, -1, 0, 1, i128::MAX] {
        let value = I64F64::from_scaled_i128(raw);
        assert_eq!(value.to_le_bytes(), raw.to_le_bytes());
        assert_eq!(I64F64::from_le_bytes(value.to_le_bytes()), value);
    }
    assert_eq!(I64F64::MIN.checked_neg(), Err(MathError::Overflow));
    assert_eq!(
        I64F64::MAX.checked_add(I64F64::QUANTUM),
        Err(MathError::Overflow)
    );
    assert_eq!(
        I64F64::MIN.checked_sub(I64F64::QUANTUM),
        Err(MathError::Overflow)
    );
}

fn ratio_oracle(
    numerator: i128,
    denominator: u128,
    direction: DirectedRounding,
) -> Result<i128, MathError> {
    let scaled = IBig::from(numerator) << 64;
    let (mut quotient, remainder) = scaled.div_rem(IBig::from(denominator));
    if !remainder.is_zero() {
        match direction {
            DirectedRounding::Floor if remainder.is_negative() => quotient -= IBig::ONE,
            DirectedRounding::Ceil if remainder.sign() == Sign::Positive => quotient += IBig::ONE,
            _ => {}
        }
    }
    i128::try_from(quotient).map_err(|_| MathError::Overflow)
}

#[test]
fn signed_ratio_accepts_full_unsigned_denominators_with_directed_rounding() {
    let numerators = [
        i128::MIN,
        i128::MIN + 1,
        -(1_i128 << 64),
        -1,
        0,
        1,
        1_i128 << 64,
        i128::MAX,
    ];
    let denominators = [
        1,
        3,
        (1_u128 << 64) - 1,
        1_u128 << 64,
        i128::MAX as u128,
        1_u128 << 127,
        u128::MAX,
    ];
    for numerator in numerators {
        for denominator in denominators {
            for direction in [DirectedRounding::Floor, DirectedRounding::Ceil] {
                assert_eq!(
                    I64F64::checked_from_ratio_round(
                        numerator,
                        NonZeroU128::new(denominator).unwrap(),
                        direction
                    )
                    .map(I64F64::to_scaled_i128),
                    ratio_oracle(numerator, denominator, direction),
                    "{numerator}/{denominator} {direction:?}"
                );
            }
        }
    }
    // Full-width random denominators exercise the unsigned high half as well
    // as tiny ratios whose floor/ceiling straddle zero.
    let mut state = 0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645_u128;
    for _ in 0..4_000 {
        state = state
            .wrapping_mul(0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645)
            .wrapping_add(1);
        let numerator = state as i128;
        state = state.rotate_left(47).wrapping_add(0x1405_7b7e_f767_814f);
        let denominator = state.max(1);
        for direction in [DirectedRounding::Floor, DirectedRounding::Ceil] {
            assert_eq!(
                I64F64::checked_from_ratio_round(
                    numerator,
                    NonZeroU128::new(denominator).unwrap(),
                    direction
                )
                .map(I64F64::to_scaled_i128),
                ratio_oracle(numerator, denominator, direction)
            );
        }
    }
}

//! Independent arbitrary-precision checks of the public scaled drivers.
use core::{mem::size_of, num::NonZeroU128};
use dashu::{
    base::{DivRem, Signed},
    integer::{IBig, UBig},
};
use mayf_fixed_drivers::{I64Scaled, U256Scaled};
use mayf_fixed_number::{
    DirectedRounding, FixedNumber, IntegralRounding, MathError, SignedFixed, UnsignedFixed,
};

fn signed_oracle(
    numerator: IBig,
    mut denominator: IBig,
    direction: DirectedRounding,
) -> Result<i64, MathError> {
    if denominator == IBig::ZERO {
        return Err(MathError::DivisionByZero);
    }
    let numerator = if denominator.is_negative() {
        denominator = -denominator;
        -numerator
    } else {
        numerator
    };
    let negative = numerator.is_negative();
    let (mut quotient, remainder) = numerator.div_rem(denominator);
    if remainder != IBig::ZERO {
        match direction {
            DirectedRounding::Floor if negative => quotient -= IBig::ONE,
            DirectedRounding::Ceil if !negative => quotient += IBig::ONE,
            _ => {}
        }
    }
    i64::try_from(quotient).map_err(|_| MathError::Overflow)
}

fn unsigned_oracle(
    numerator: UBig,
    denominator: UBig,
    direction: DirectedRounding,
) -> Result<UBig, MathError> {
    if denominator == UBig::ZERO {
        return Err(MathError::DivisionByZero);
    }
    let (mut quotient, remainder) = numerator.div_rem(denominator);
    if remainder != UBig::ZERO && direction == DirectedRounding::Ceil {
        quotient += UBig::ONE;
    }
    if quotient >= UBig::ONE << 256 {
        Err(MathError::Overflow)
    } else {
        Ok(quotient)
    }
}

fn raw<const SCALE: u64>(value: U256Scaled<SCALE>) -> UBig {
    UBig::from_le_bytes(&value.to_le_bytes())
}

fn wide<const SCALE: u64>(value: &UBig) -> U256Scaled<SCALE> {
    let mut bytes = [0_u8; 32];
    let input = value.to_le_bytes();
    bytes[..input.len()].copy_from_slice(&input);
    U256Scaled::from_le_bytes(bytes)
}

fn signed_cases<const SCALE: u64>() {
    let values = [
        i64::MIN,
        i64::MIN + 1,
        -10,
        -3,
        -1,
        0,
        1,
        3,
        10,
        i64::MAX - 1,
        i64::MAX,
    ];
    for left in values {
        let l = I64Scaled::<SCALE>::from_scaled_i64(left);
        assert_eq!(I64Scaled::from_le_bytes(l.to_le_bytes()), l);
        assert_eq!(
            l.checked_neg().map(I64Scaled::to_scaled_i64),
            left.checked_neg().ok_or(MathError::Overflow)
        );
        for right in values {
            let r = I64Scaled::from_scaled_i64(right);
            assert_eq!(
                l.checked_add(r).map(I64Scaled::to_scaled_i64),
                left.checked_add(right).ok_or(MathError::Overflow)
            );
            assert_eq!(
                l.checked_sub(r).map(I64Scaled::to_scaled_i64),
                left.checked_sub(right).ok_or(MathError::Overflow)
            );
            for direction in [DirectedRounding::Floor, DirectedRounding::Ceil] {
                assert_eq!(
                    l.checked_mul_round(r, direction)
                        .map(I64Scaled::to_scaled_i64),
                    signed_oracle(
                        IBig::from(left) * IBig::from(right),
                        IBig::from(SCALE),
                        direction
                    )
                );
                assert_eq!(
                    l.checked_div_round(r, direction)
                        .map(I64Scaled::to_scaled_i64),
                    signed_oracle(
                        IBig::from(left) * IBig::from(SCALE),
                        IBig::from(right),
                        direction
                    )
                );
                for divisor in values {
                    let d = I64Scaled::from_scaled_i64(divisor);
                    assert_eq!(
                        l.checked_mul_div_round(r, d, direction)
                            .map(I64Scaled::to_scaled_i64),
                        signed_oracle(
                            IBig::from(left) * IBig::from(right),
                            IBig::from(divisor),
                            direction
                        ),
                        "scale={SCALE} {left}*{right}/{divisor} {direction:?}"
                    );
                }
            }
            for divisor in [0, 1, 3, i64::MAX] {
                if left < 0 || right < 0 {
                    continue;
                }
                let d = I64Scaled::from_scaled_i64(divisor);
                let expected = signed_oracle(
                    IBig::from(left) * IBig::from(right),
                    IBig::from(divisor),
                    DirectedRounding::Floor,
                )
                .and_then(|lo| {
                    signed_oracle(
                        IBig::from(left) * IBig::from(right),
                        IBig::from(divisor),
                        DirectedRounding::Ceil,
                    )
                    .map(|hi| (lo, hi))
                });
                let actual = (l.bounded() * r / d)
                    .bounds()
                    .map(|bounds| bounds.into_raw().into_edges())
                    .map(|(lo, hi)| (lo.to_scaled_i64(), hi.to_scaled_i64()));
                assert_eq!(actual, expected);
            }
        }
    }
    for numerator in [i128::MIN, -1, 0, 1, i128::MAX] {
        for denominator in [1, 3, 1_u128 << 64, 1_u128 << 127, u128::MAX] {
            for direction in [DirectedRounding::Floor, DirectedRounding::Ceil] {
                assert_eq!(
                    I64Scaled::<SCALE>::checked_from_ratio_round(
                        numerator,
                        NonZeroU128::new(denominator).unwrap(),
                        direction
                    )
                    .map(I64Scaled::to_scaled_i64),
                    signed_oracle(
                        IBig::from(numerator) * IBig::from(SCALE),
                        IBig::from(denominator),
                        direction
                    )
                );
            }
        }
    }
}

#[test]
fn signed_rounding_and_full_width_ratios_match_exact_arithmetic() {
    signed_cases::<1>();
    signed_cases::<3>();
    signed_cases::<10>();
    signed_cases::<1_000_000_000_000>();
    signed_cases::<{ i64::MAX as u64 }>();
}

fn check_wide<const SCALE: u64>(left: &UBig, right: &UBig, divisor: &UBig) {
    let l = wide::<SCALE>(left);
    let r = wide::<SCALE>(right);
    let d = wide::<SCALE>(divisor);
    for direction in [DirectedRounding::Floor, DirectedRounding::Ceil] {
        assert_eq!(
            l.checked_mul_round(r, direction).map(raw),
            unsigned_oracle(left * right, UBig::from(SCALE), direction)
        );
        assert_eq!(
            l.checked_div_round(r, direction).map(raw),
            unsigned_oracle(left * UBig::from(SCALE), right.clone(), direction)
        );
        assert_eq!(
            l.checked_mul_div_round(r, d, direction).map(raw),
            unsigned_oracle(left * right, divisor.clone(), direction),
            "scale={SCALE} {left}*{right}/{divisor} {direction:?}"
        );
    }
    let expected = unsigned_oracle(left * right, divisor.clone(), DirectedRounding::Floor)
        .and_then(|lo| {
            unsigned_oracle(left * right, divisor.clone(), DirectedRounding::Ceil)
                .map(|hi| (lo, hi))
        });
    let actual = (l.bounded() * r / d)
        .bounds()
        .map(|bounds| bounds.into_raw().into_edges())
        .map(|(lo, hi)| (raw(lo), raw(hi)));
    assert_eq!(actual, expected);
}

fn wide_cases<const SCALE: u64>() {
    let values = [
        UBig::ZERO,
        UBig::ONE,
        UBig::from(SCALE),
        (UBig::ONE << 128) - UBig::ONE,
        UBig::ONE << 128,
        UBig::ONE << 255,
        (UBig::ONE << 256) - UBig::ONE,
    ];
    for left in &values {
        assert_eq!(raw(wide::<SCALE>(left)), *left);
        for right in &values {
            for divisor in &values {
                check_wide::<SCALE>(left, right, divisor);
            }
        }
    }
    for numerator in [0, 1, i128::MAX] {
        for denominator in [1, 3, 1_u128 << 127, u128::MAX] {
            for direction in [DirectedRounding::Floor, DirectedRounding::Ceil] {
                assert_eq!(
                    U256Scaled::<SCALE>::checked_from_ratio_round(
                        numerator,
                        NonZeroU128::new(denominator).unwrap(),
                        direction
                    )
                    .map(raw),
                    unsigned_oracle(
                        UBig::from(numerator as u128) * UBig::from(SCALE),
                        UBig::from(denominator),
                        direction
                    )
                );
            }
        }
    }
    let mut state = 0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645_u128;
    for index in 0..200 {
        let mut next = || {
            state = state
                .wrapping_mul(0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645)
                .wrapping_add(1);
            let high = UBig::from(state) << 128;
            state = state.rotate_left(47).wrapping_add(0x1405_7b7e_f767_814f);
            (high + UBig::from(state)) >> [0, 64, 128, 192][index % 4]
        };
        check_wide::<SCALE>(&next(), &next(), &next());
    }
}

#[test]
fn wide_products_ratios_and_bounds_match_exact_arithmetic() {
    wide_cases::<1>();
    wide_cases::<10>();
    wide_cases::<1_000_000_000_000>();
    wide_cases::<{ u64::MAX }>();
}

#[test]
fn scaled_storage_capabilities_and_integral_limits_are_checked() {
    fn unsigned<N: UnsignedFixed>() {}
    unsigned::<U256Scaled<10>>();
    assert_eq!(size_of::<I64Scaled<10>>(), 8);
    assert_eq!(size_of::<U256Scaled<10>>(), 32);
    type Small = I64Scaled<10>;
    type Wide = U256Scaled<10>;
    assert_eq!(Small::ONE.to_scaled_i64(), 10);
    assert_eq!(Small::QUANTUM.to_scaled_i64(), 1);
    assert_eq!(Small::from_scaled_i64(29).checked_floor_to_u64(), Ok(2));
    assert_eq!(
        Small::from_scaled_i64(-1).checked_floor_to_u64(),
        Err(MathError::NegativeToUnsigned)
    );
    assert_eq!(
        Small::checked_from_u64((i64::MAX / 10) as u64 + 1),
        Err(MathError::Overflow)
    );
    assert_eq!(
        Wide::checked_from_i128(-1),
        Err(MathError::NegativeToUnsigned)
    );
    assert_eq!(
        Wide::ZERO.checked_sub(Wide::QUANTUM),
        Err(MathError::Overflow)
    );
    assert_eq!(
        Wide::MAX.checked_add(Wide::QUANTUM),
        Err(MathError::Overflow)
    );
    assert_eq!(Wide::MAX.checked_to_scaled_u128(), Err(MathError::Overflow));
    for value in [0, 1, u128::MAX] {
        assert_eq!(
            Wide::from_scaled_u128(value).checked_to_scaled_u128(),
            Ok(value)
        );
    }
    let maximum = Wide::from(u64::MAX)
        .checked_add(Wide::from_scaled_u128(9))
        .unwrap();
    assert_eq!(maximum.checked_floor_to_u64(), Ok(u64::MAX));
    assert_eq!(
        maximum
            .checked_add(Wide::QUANTUM)
            .unwrap()
            .checked_floor_to_u64(),
        Err(MathError::Overflow)
    );
    assert_eq!(Wide::MAX.checked_floor_to_u64(), Err(MathError::Overflow));
    let n = Wide::from(u64::MAX);
    assert_eq!(
        raw(n.checked_mul_floor(n).unwrap()),
        UBig::from(u64::MAX) * UBig::from(u64::MAX) * UBig::from(10_u8)
    );
    assert_eq!(
        Wide::MAX.checked_mul_div_floor(Wide::MAX, Wide::MAX),
        Ok(Wide::MAX)
    );
    assert_eq!(
        Small::MAX.checked_mul_div_floor(Small::MAX, Small::MAX),
        Ok(Small::MAX)
    );
    // (MAX-1)^2/(MAX-2) = MAX + 1/(MAX-2): only the ceiling overflows.
    let left = Wide::MAX.checked_sub(Wide::QUANTUM).unwrap();
    let divisor = left.checked_sub(Wide::QUANTUM).unwrap();
    assert_eq!(left.checked_mul_div_floor(left, divisor), Ok(Wide::MAX));
    assert_eq!(
        left.checked_mul_div_ceil(left, divisor),
        Err(MathError::Overflow)
    );
}

#[cfg(feature = "reference")]
#[test]
fn public_reference_types_support_independent_host_arithmetic() {
    use mayf_fixed_drivers::reference::{DashuFixed12, IBig as PublicInt, UBig as PublicUint};
    assert_eq!(PublicInt::from(-7), -PublicInt::from(7));
    assert_eq!(
        (PublicUint::ONE << 300) / (PublicUint::ONE << 200),
        PublicUint::ONE << 100
    );
    assert_eq!(
        DashuFixed12::ONE
            .checked_div_floor(DashuFixed12::from(2))
            .unwrap()
            .to_scaled_i128(),
        500_000_000_000
    );
}

//! Exact limb kernels shared by the fixed-point drivers.

use mayf_fixed_number::{DirectedRounding, MathError, MathResult};
const RAW_ONE: i128 = 1i128 << 64;

/// Nonnegative raw product/division with one quotient and an exact remainder
/// flag. The product occupies 256 bits; the Q64 divisor uses a limb splice.
pub(crate) fn unsigned_mul_div_floor(
    left: u128,
    right: u128,
    divisor: u128,
) -> MathResult<mayf_fixed_number::driver::Floored<u128>> {
    if divisor == 0 {
        return Err(MathError::DivisionByZero);
    }
    let (hi, lo) = wide_mul_u128(left, right);
    if hi >= divisor {
        return Err(MathError::Overflow);
    }
    let (value, remainder) = if divisor == RAW_ONE as u128 {
        ((hi << 64) | (lo >> 64), lo & ((1_u128 << 64) - 1))
    } else {
        div_rem_wide(hi, lo, divisor)
    };
    Ok(mayf_fixed_number::driver::Floored {
        value,
        inexact: remainder != 0,
    })
}

/// Signed raw product/division with one final directed rounding. Private signed
/// kernels serve signed driver operations and exponential range reduction.
pub(crate) fn muldiv_raw(
    left: i128,
    right: i128,
    divisor: i128,
    rounding: DirectedRounding,
) -> MathResult<i128> {
    if divisor == 0 {
        return Err(MathError::DivisionByZero);
    }

    // Fixed-point multiplication always divides the exact 256-bit product by
    // `2^64`. That quotient is just a limb splice and the remainder is the low
    // 64 bits; sending this overwhelmingly common case through 128 rounds of
    // restoring division is needlessly expensive on SBF.
    if divisor == RAW_ONE {
        return mul_q64_raw(left, right, rounding);
    }

    // A fixed-point integral divisor is `integer * 2^64`. First splice the
    // exact wide product across that `2^64` boundary, then divide its resulting
    // 128-bit magnitude by the small integer using four native 64/32-bit
    // divisions. This covers both ordinary division and fused recurrence steps
    // `(term * x) / integer` without separately rounding the product.
    if divisor & (RAW_ONE - 1) == 0 {
        let integral_divisor = divisor / RAW_ONE;
        let divisor_magnitude = integral_divisor.unsigned_abs();
        if integral_divisor != 0
            && divisor_magnitude <= u128::from(u32::MAX)
            && let Some(result) = muldiv_q64_by_small_integral(
                left,
                right,
                integral_divisor,
                divisor_magnitude as u32,
                rounding,
            )
        {
            return result;
        }
    }

    muldiv_raw_reference(left, right, divisor, rounding)
}

/// Q64.64 multiplication with a single final directed rounding.
///
/// The exact unsigned product is `hi * 2^128 + lo`. Dividing it by `2^64`
/// yields `(hi << 64) | (lo >> 64)`, with `lo mod 2^64` as the remainder.
/// The same `hi < 2^64` condition used by the generic divider proves that the
/// unsigned quotient fits in 128 bits.
fn mul_q64_raw(left: i128, right: i128, rounding: DirectedRounding) -> MathResult<i128> {
    const FRACTION_MASK: u128 = (1u128 << 64) - 1;

    let negative = (left < 0) != (right < 0);
    let (hi, lo) = wide_mul_u128(left.unsigned_abs(), right.unsigned_abs());
    if hi >= RAW_ONE as u128 {
        return Err(MathError::Overflow);
    }

    let mut quotient = (hi << 64) | (lo >> 64);
    let remainder = lo & FRACTION_MASK;
    let rounds_away_from_zero = match rounding {
        DirectedRounding::Floor => negative,
        DirectedRounding::Ceil => !negative,
    };
    if remainder != 0 && rounds_away_from_zero {
        quotient = quotient.checked_add(1).ok_or(MathError::Overflow)?;
    }

    signed_from_magnitude(quotient, negative)
}

/// Compute `(left * right) / (divisor * 2^64)` when the post-Q64 product fits
/// in `u128` and `|divisor|` fits in `u32`.
///
/// Returns `None` when the intermediate magnitude after division by `2^64`
/// exceeds 128 bits. The final quotient may still fit after division by the
/// small integer, so callers must fall back to the generic 256/128 divider in
/// that case rather than reporting overflow.
fn muldiv_q64_by_small_integral(
    left: i128,
    right: i128,
    divisor: i128,
    divisor_magnitude: u32,
    rounding: DirectedRounding,
) -> Option<MathResult<i128>> {
    const FRACTION_MASK: u128 = (1u128 << 64) - 1;

    debug_assert_ne!(divisor, 0);
    debug_assert_eq!(divisor.unsigned_abs(), u128::from(divisor_magnitude));

    let negative = ((left < 0) != (right < 0)) != (divisor < 0);
    let (hi, lo) = wide_mul_u128(left.unsigned_abs(), right.unsigned_abs());
    if hi >= RAW_ONE as u128 {
        return None;
    }

    let post_q64_magnitude = (hi << 64) | (lo >> 64);
    let fractional_remainder = lo & FRACTION_MASK;
    let (mut quotient, integral_remainder) =
        div_rem_u128_by_u32(post_q64_magnitude, divisor_magnitude);
    let inexact = fractional_remainder != 0 || integral_remainder != 0;
    let rounds_away_from_zero = match rounding {
        DirectedRounding::Floor => negative,
        DirectedRounding::Ceil => !negative,
    };
    if inexact && rounds_away_from_zero {
        let Some(rounded) = quotient.checked_add(1) else {
            return Some(Err(MathError::Overflow));
        };
        quotient = rounded;
    }

    Some(signed_from_magnitude(quotient, negative))
}

/// Four-limb unsigned `u128 / u32` division.
///
/// Each step's dividend is at most `(divisor - 1) * 2^32 + u32::MAX`, so it
/// fits in `u64`. This deliberately avoids compiler-emitted 128-bit division
/// helpers on SBF.
fn div_rem_u128_by_u32(value: u128, divisor: u32) -> (u128, u32) {
    debug_assert_ne!(divisor, 0);

    let divisor = u64::from(divisor);
    let mut quotient = 0u128;
    let mut remainder = 0u64;
    let mut limb = 4u32;
    while limb > 0 {
        limb -= 1;
        let shift = limb * 32;
        let digit = ((value >> shift) & u128::from(u32::MAX)) as u64;
        let dividend = (remainder << 32) | digit;
        quotient = (quotient << 32) | u128::from(dividend / divisor);
        remainder = dividend % divisor;
    }

    (quotient, remainder as u32)
}

/// Fully generic reference/fallback implementation.
///
/// Keeping this path separate makes the optimized algebraic cases directly
/// comparable against the original exact 256/128-bit algorithm in tests.
fn muldiv_raw_reference(
    left: i128,
    right: i128,
    divisor: i128,
    rounding: DirectedRounding,
) -> MathResult<i128> {
    if divisor == 0 {
        return Err(MathError::DivisionByZero);
    }

    let negative = ((left < 0) != (right < 0)) != (divisor < 0);
    let (hi, lo) = wide_mul_u128(left.unsigned_abs(), right.unsigned_abs());
    let divisor_magnitude = divisor.unsigned_abs();
    if hi >= divisor_magnitude {
        return Err(MathError::Overflow);
    }

    let (mut quotient, remainder) = div_rem_wide(hi, lo, divisor_magnitude);
    let rounds_away_from_zero = match rounding {
        DirectedRounding::Floor => negative,
        DirectedRounding::Ceil => !negative,
    };
    if remainder != 0 && rounds_away_from_zero {
        quotient = quotient.checked_add(1).ok_or(MathError::Overflow)?;
    }

    signed_from_magnitude(quotient, negative)
}

/// Full 256-bit product of two unsigned 128-bit values via 64-bit limbs.
pub(crate) fn wide_mul_u128(a: u128, b: u128) -> (u128, u128) {
    const MASK: u128 = (1u128 << 64) - 1;

    let (a_hi, a_lo) = (a >> 64, a & MASK);
    let (b_hi, b_lo) = (b >> 64, b & MASK);

    let ll = a_lo * b_lo;
    let lh = a_lo * b_hi;
    let hl = a_hi * b_lo;
    let hh = a_hi * b_hi;

    let mid = (ll >> 64) + (lh & MASK) + (hl & MASK);
    let lo = (ll & MASK) | (mid << 64);
    let hi = hh + (lh >> 64) + (hl >> 64) + (mid >> 64);

    (hi, lo)
}

/// Divide the 256-bit value `hi * 2^128 + lo` by `divisor`.
///
/// Precondition: `hi < divisor`, which guarantees the quotient fits in 128
/// bits.
///
/// Schoolbook long division over 64-bit limbs, so the work is a handful of
/// native `u128` divisions rather than one iteration per quotient bit. A
/// bit-at-a-time loop costs the same 128 iterations for every input, and on
/// SBF that made a fixed-point division roughly thirty times a multiplication.
/// `div_rem_wide_reference` in the tests is the bit-at-a-time version, kept as
/// the oracle this must agree with everywhere.
pub(crate) fn div_rem_wide(hi: u128, lo: u128, divisor: u128) -> (u128, u128) {
    debug_assert!(hi < divisor);

    // The common case: the numerator already fits the native division.
    if hi == 0 {
        return (lo / divisor, lo % divisor);
    }

    // A divisor inside one limb needs no normalization. `hi < divisor` makes
    // the most significant quotient limb zero, so two divisions suffice.
    if divisor >> 64 == 0 {
        let (high_quotient, remainder) = div_rem_limb(hi, lo >> 64, divisor);
        let (low_quotient, remainder) = div_rem_limb(remainder, lo & LOW_LIMB_MASK, divisor);
        return ((high_quotient << 64) | low_quotient, remainder);
    }

    // A divisor that is a whole number of fixed-point units has 64 low zero
    // bits, so it divides the numerator's limbs directly. Dividing by a count
    // is common enough to be worth naming, and it skips both normalization and
    // the trial-quotient correction.
    if divisor & LOW_LIMB_MASK == 0 {
        let units = divisor >> 64;
        let (high_quotient, remainder) = div_rem_limb(hi >> 64, hi & LOW_LIMB_MASK, units);
        let (low_quotient, remainder) = div_rem_limb(remainder, lo >> 64, units);
        return (
            (high_quotient << 64) | low_quotient,
            (remainder << 64) | (lo & LOW_LIMB_MASK),
        );
    }

    // Normalize so the divisor's most significant bit is set, which is what
    // bounds the trial quotient below to at most two too large.
    let shift = divisor.leading_zeros();
    let divisor_normalized = divisor << shift;
    let (numerator_high, numerator_low) = if shift == 0 {
        (hi, lo)
    } else {
        ((hi << shift) | (lo >> (128 - shift)), lo << shift)
    };

    let divisor_high_limb = (divisor_normalized >> 64) as u64;
    let (high_quotient, remainder) = div_rem_three_by_two(
        (numerator_high >> 64) as u64,
        numerator_high as u64,
        (numerator_low >> 64) as u64,
        divisor_normalized,
        divisor_high_limb,
    );
    let (low_quotient, remainder) = div_rem_three_by_two(
        (remainder >> 64) as u64,
        remainder as u64,
        numerator_low as u64,
        divisor_normalized,
        divisor_high_limb,
    );

    (
        (u128::from(high_quotient) << 64) | u128::from(low_quotient),
        remainder >> shift,
    )
}

const LOW_LIMB_MASK: u128 = (1u128 << 64) - 1;

/// Divide `remainder * 2^64 + limb` by a divisor that fits in one limb.
fn div_rem_limb(remainder: u128, limb: u128, divisor: u128) -> (u128, u128) {
    debug_assert!(divisor >> 64 == 0);
    debug_assert!(remainder < divisor);

    let numerator = (remainder << 64) | limb;
    (numerator / divisor, numerator % divisor)
}

/// Divide the three limbs `u2:u1:u0` by a normalized two-limb `divisor`.
///
/// Preconditions: `divisor >= 2^127` and `u2:u1 < divisor`, which together
/// make the quotient a single limb. The trial quotient taken from the
/// divisor's high limb is never below the true one and never more than two
/// above it, so correcting it downward terminates immediately.
fn div_rem_three_by_two(
    u2: u64,
    u1: u64,
    u0: u64,
    divisor: u128,
    divisor_high_limb: u64,
) -> (u64, u128) {
    debug_assert!(divisor.leading_zeros() == 0);
    debug_assert!(((u128::from(u2) << 64) | u128::from(u1)) < divisor);

    let leading = (u128::from(u2) << 64) | u128::from(u1);
    let mut quotient = if u2 == divisor_high_limb {
        u64::MAX
    } else {
        (leading / u128::from(divisor_high_limb)) as u64
    };

    // Subtracting the trial product across all three limbs decides whether the
    // estimate was right: a borrow out of the top limb means it was too large.
    // The loop is bounded rather than open, so a wrong estimate can never spin
    // or wrap the quotient past zero.
    let numerator_low = (u128::from(u1) << 64) | u128::from(u0);
    for _ in 0..=MAX_TRIAL_QUOTIENT_CORRECTIONS {
        let (product_high, product_low) = wide_mul_u128(divisor, u128::from(quotient));
        let (remainder, borrow) = numerator_low.overflowing_sub(product_low);
        let remainder_high = u128::from(u2)
            .wrapping_sub(product_high)
            .wrapping_sub(u128::from(borrow));
        if remainder_high == 0 && remainder < divisor {
            return (quotient, remainder);
        }
        quotient -= 1;
    }

    unreachable!("a normalized trial quotient is at most two above the true one")
}

/// How far a trial quotient taken from a normalized divisor's high limb can
/// exceed the true one. This is Knuth's bound for algorithm D.
const MAX_TRIAL_QUOTIENT_CORRECTIONS: u32 = 2;

pub(crate) fn signed_from_magnitude(magnitude: u128, negative: bool) -> MathResult<i128> {
    const I128_MIN_MAGNITUDE: u128 = 1u128 << 127;

    if negative {
        if magnitude > I128_MIN_MAGNITUDE {
            return Err(MathError::Overflow);
        }
        if magnitude == I128_MIN_MAGNITUDE {
            return Ok(i128::MIN);
        }
        Ok(-(magnitude as i128))
    } else {
        i128::try_from(magnitude).map_err(|_| MathError::Overflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROUNDINGS: [DirectedRounding; 2] = [DirectedRounding::Floor, DirectedRounding::Ceil];

    fn next_state(state: &mut u128) -> u128 {
        *state = state
            .wrapping_mul(0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645)
            .wrapping_add(0x1405_7b7e_f767_814f);
        *state
    }

    fn next_varying_i128(state: &mut u128) -> i128 {
        let value = next_state(state) as i128;
        let shift = (next_state(state) >> 121) as u32 % 128;
        value >> shift
    }

    #[test]
    fn wide_mul_matches_known_products() {
        assert_eq!(wide_mul_u128(0, u128::MAX), (0, 0));
        assert_eq!(wide_mul_u128(1, u128::MAX), (0, u128::MAX));
        assert_eq!(wide_mul_u128(u128::MAX, u128::MAX), (u128::MAX - 1, 1));
        assert_eq!(wide_mul_u128(1u128 << 64, 1u128 << 64), (1, 0));
        assert_eq!(
            wide_mul_u128(u128::MAX, 2),
            (1, u128::MAX - 1),
            "2 * (2^128 - 1) = 2^129 - 2"
        );
    }

    #[test]
    fn wide_division_reconstructs_the_numerator() {
        let cases = [
            (0u128, 7u128, 3u128),
            (0, u128::MAX, 1),
            (0, u128::MAX, u128::MAX),
            (1, 0, 3),
            (4, u128::MAX, 5),
            (u128::MAX - 1, u128::MAX, u128::MAX),
            (5, 42, 1u128 << 127),
            ((1u128 << 127) - 1, u128::MAX, u128::MAX),
        ];

        for (hi, lo, divisor) in cases {
            assert!(hi < divisor, "case violates the quotient-fits precondition");
            let (quotient, remainder) = div_rem_wide(hi, lo, divisor);
            assert!(remainder < divisor);

            let (product_hi, product_lo) = wide_mul_u128(quotient, divisor);
            let (sum_lo, carry) = product_lo.overflowing_add(remainder);
            let sum_hi = product_hi + u128::from(carry);
            assert_eq!((sum_hi, sum_lo), (hi, lo), "q * d + r must equal n");
        }
    }

    /// Restoring binary long division, one iteration per quotient bit.
    ///
    /// This is what `div_rem_wide` used to be. It is obviously correct and
    /// obviously slow, which makes it the right oracle for the limb version.
    fn div_rem_wide_reference(hi: u128, lo: u128, divisor: u128) -> (u128, u128) {
        let mut remainder = hi;
        let mut quotient = 0u128;

        let mut bit = 128u32;
        while bit > 0 {
            bit -= 1;
            let carry = remainder >> 127;
            remainder = (remainder << 1) | ((lo >> bit) & 1);
            quotient <<= 1;
            if carry == 1 || remainder >= divisor {
                remainder = remainder.wrapping_sub(divisor);
                quotient |= 1;
            }
        }

        (quotient, remainder)
    }

    #[test]
    fn wide_division_agrees_with_the_bit_at_a_time_reference() {
        // Boundaries where limb division changes shape: a single-limb divisor,
        // an already normalized one, one that needs the maximum shift, and the
        // trial-quotient cases that need a downward correction.
        let interesting = [
            1u128,
            2,
            3,
            u128::from(u32::MAX),
            u128::from(u64::MAX),
            1u128 << 64,
            (1u128 << 64) + 1,
            u128::MAX >> 1,
            1u128 << 127,
            (1u128 << 127) + 1,
            u128::MAX - 1,
            u128::MAX,
        ];
        for divisor in interesting {
            for hi in interesting.iter().copied().filter(|hi| *hi < divisor) {
                for lo in interesting {
                    assert_eq!(
                        div_rem_wide(hi, lo, divisor),
                        div_rem_wide_reference(hi, lo, divisor),
                        "{hi} * 2^128 + {lo} / {divisor}"
                    );
                }
            }
            for lo in interesting {
                assert_eq!(
                    div_rem_wide(0, lo, divisor),
                    div_rem_wide_reference(0, lo, divisor),
                );
            }
        }

        // A wide pseudo-random sweep, biased toward divisors near each limb
        // boundary because that is where the trial quotient is loosest.
        let mut state = 0x243f_6a88_85a3_08d3u64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for _ in 0..20_000 {
            let divisor = match next() % 4 {
                0 => u128::from(next()),
                1 => (u128::from(next()) << 64) | u128::from(next()),
                2 => u128::MAX - u128::from(next() % 1_024),
                _ => (1u128 << 127) | u128::from(next()),
            }
            .max(1);
            let hi = ((u128::from(next()) << 64) | u128::from(next())) % divisor;
            let lo = (u128::from(next()) << 64) | u128::from(next());
            assert_eq!(
                div_rem_wide(hi, lo, divisor),
                div_rem_wide_reference(hi, lo, divisor),
                "{hi} * 2^128 + {lo} / {divisor}"
            );
        }
    }

    #[test]
    fn four_limb_small_division_matches_native_arithmetic_exhaustively() {
        for value in 0u128..=1_024 {
            for divisor in 1u32..=67 {
                assert_eq!(
                    div_rem_u128_by_u32(value, divisor),
                    (
                        value / u128::from(divisor),
                        (value % u128::from(divisor)) as u32
                    ),
                    "{value} / {divisor}"
                );
            }
        }

        let boundary_values = [
            0,
            1,
            u128::from(u32::MAX),
            1u128 << 32,
            1u128 << 64,
            1u128 << 96,
            1u128 << 127,
            u128::MAX - 1,
            u128::MAX,
        ];
        let boundary_divisors = [1, 2, 3, 64, 67, u32::MAX - 1, u32::MAX];
        for value in boundary_values {
            for divisor in boundary_divisors {
                assert_eq!(
                    div_rem_u128_by_u32(value, divisor),
                    (
                        value / u128::from(divisor),
                        (value % u128::from(divisor)) as u32
                    ),
                    "{value} / {divisor}"
                );
            }
        }
    }

    fn assert_q64_multiply_matches_reference(left: i128, right: i128) {
        for rounding in ROUNDINGS {
            assert_eq!(
                mul_q64_raw(left, right, rounding),
                muldiv_raw_reference(left, right, RAW_ONE, rounding),
                "Q64 multiply {left} * {right} with {rounding:?}"
            );
        }
    }

    #[test]
    fn q64_multiply_fast_path_matches_reference_exhaustively_near_grid_points() {
        for left_whole in -8i128..=8 {
            for left_fraction in -4i128..=4 {
                let left = left_whole * RAW_ONE + left_fraction;
                for right_whole in -8i128..=8 {
                    for right_fraction in -4i128..=4 {
                        let right = right_whole * RAW_ONE + right_fraction;
                        assert_q64_multiply_matches_reference(left, right);
                    }
                }
            }
        }
    }

    #[test]
    fn q64_multiply_fast_path_matches_reference_at_boundaries_and_randomly() {
        let boundaries = [
            i128::MIN,
            i128::MIN + 1,
            -(1i128 << 100),
            -RAW_ONE - 1,
            -RAW_ONE,
            -RAW_ONE + 1,
            -1,
            0,
            1,
            RAW_ONE - 1,
            RAW_ONE,
            RAW_ONE + 1,
            1i128 << 100,
            i128::MAX - 1,
            i128::MAX,
        ];
        for left in boundaries {
            for right in boundaries {
                assert_q64_multiply_matches_reference(left, right);
            }
        }

        let mut state = 0x853c_49e6_748f_ea9b_da3e_39cb_94b9_5bdbu128;
        for _ in 0..20_000 {
            assert_q64_multiply_matches_reference(
                next_varying_i128(&mut state),
                next_varying_i128(&mut state),
            );
        }
    }

    fn assert_small_integral_division_matches_reference(value: i128, divisor: i128) {
        let scaled_divisor = divisor
            .checked_mul(RAW_ONE)
            .expect("test divisor must fit the fixed grid");
        for rounding in ROUNDINGS {
            assert_eq!(
                muldiv_q64_by_small_integral(
                    value,
                    RAW_ONE,
                    divisor,
                    divisor.unsigned_abs() as u32,
                    rounding,
                ),
                Some(muldiv_raw_reference(
                    value,
                    RAW_ONE,
                    scaled_divisor,
                    rounding
                )),
                "Q64 division {value} / {divisor} with {rounding:?}"
            );
            assert_eq!(
                muldiv_raw(value, RAW_ONE, scaled_divisor, rounding),
                muldiv_raw_reference(value, RAW_ONE, scaled_divisor, rounding),
                "dispatched Q64 division {value} / {divisor} with {rounding:?}"
            );
        }
    }

    fn assert_fused_small_integral_division_matches_reference(
        left: i128,
        right: i128,
        divisor: i128,
    ) {
        let scaled_divisor = divisor
            .checked_mul(RAW_ONE)
            .expect("test divisor must fit the fixed grid");
        for rounding in ROUNDINGS {
            let reference = muldiv_raw_reference(left, right, scaled_divisor, rounding);
            if let Some(fast) = muldiv_q64_by_small_integral(
                left,
                right,
                divisor,
                divisor.unsigned_abs() as u32,
                rounding,
            ) {
                assert_eq!(
                    fast, reference,
                    "fused Q64 division {left} * {right} / {divisor} with {rounding:?}"
                );
            }
            assert_eq!(
                muldiv_raw(left, right, scaled_divisor, rounding),
                reference,
                "dispatched fused Q64 division {left} * {right} / {divisor} with {rounding:?}"
            );
        }
    }

    #[test]
    fn small_integral_division_fast_path_matches_reference() {
        let boundaries = [
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
        for value in boundaries {
            for divisor in -67i128..=-1 {
                assert_small_integral_division_matches_reference(value, divisor);
            }
            for divisor in 1i128..=67 {
                assert_small_integral_division_matches_reference(value, divisor);
            }
            for divisor in [-(i128::from(u32::MAX)), i128::from(u32::MAX)] {
                assert_small_integral_division_matches_reference(value, divisor);
            }
        }

        let mut state = 0xda3e_39cb_94b9_5bdb_853c_49e6_748f_ea9bu128;
        for _ in 0..20_000 {
            let value = next_state(&mut state) as i128;
            let magnitude = (next_state(&mut state) % 67 + 1) as i128;
            let divisor = if next_state(&mut state) & 1 == 0 {
                magnitude
            } else {
                -magnitude
            };
            assert_small_integral_division_matches_reference(value, divisor);
        }
    }

    #[test]
    fn fused_small_integral_division_fast_path_matches_reference() {
        let boundaries = [
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
        for left in boundaries {
            for right in boundaries {
                for divisor in [-67i128, -3, -2, 2, 3, 67] {
                    assert_fused_small_integral_division_matches_reference(left, right, divisor);
                }
            }
        }

        let mut state = 0x4d59_5df4_d0f3_3173_1b03_734e_a44f_9e93u128;
        for _ in 0..20_000 {
            let magnitude = (next_state(&mut state) % 67 + 1) as i128;
            let divisor = if next_state(&mut state) & 1 == 0 {
                magnitude
            } else {
                -magnitude
            };
            assert_fused_small_integral_division_matches_reference(
                next_varying_i128(&mut state),
                next_varying_i128(&mut state),
                divisor,
            );
        }
    }

    /// Mathematical floor of `a / b` for any sign combination. `div_euclid`
    /// is not floor division when the divisor is negative.
    fn floor_div(a: i128, b: i128) -> i128 {
        let quotient = a / b;
        let remainder = a % b;
        if remainder != 0 && ((remainder < 0) != (b < 0)) {
            quotient - 1
        } else {
            quotient
        }
    }

    #[test]
    fn muldiv_matches_exact_small_arithmetic() {
        for left in -12i128..=12 {
            for right in -12i128..=12 {
                for divisor in [-7i128, -3, -1, 1, 2, 5, 9] {
                    let product = left * right;
                    let exact_floor = floor_div(product, divisor);
                    let exact_ceil = -floor_div(-product, divisor);

                    assert_eq!(
                        muldiv_raw(left, right, divisor, DirectedRounding::Floor),
                        Ok(exact_floor),
                        "floor {left} * {right} / {divisor}"
                    );
                    assert_eq!(
                        muldiv_raw(left, right, divisor, DirectedRounding::Ceil),
                        Ok(exact_ceil),
                        "ceil {left} * {right} / {divisor}"
                    );
                }
            }
        }
    }

    #[test]
    fn muldiv_covers_extreme_magnitudes() {
        assert_eq!(
            muldiv_raw(i128::MAX, RAW_ONE, RAW_ONE, DirectedRounding::Floor),
            Ok(i128::MAX)
        );
        assert_eq!(
            muldiv_raw(i128::MAX, i128::MAX, i128::MAX, DirectedRounding::Floor),
            Ok(i128::MAX)
        );
        assert_eq!(
            muldiv_raw(i128::MIN, RAW_ONE, RAW_ONE, DirectedRounding::Floor),
            Ok(i128::MIN)
        );
        assert_eq!(
            muldiv_raw(i128::MAX, 2, 1, DirectedRounding::Floor),
            Err(MathError::Overflow)
        );
        assert_eq!(
            muldiv_raw(1, 1, 0, DirectedRounding::Floor),
            Err(MathError::DivisionByZero)
        );
    }
}

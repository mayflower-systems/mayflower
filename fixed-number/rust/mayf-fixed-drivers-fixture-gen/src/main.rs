//! Writes the oracle fixtures that the TypeScript package
//! `@mayflower-sys/fixed-number-ts` tests against.
//!
//! Every serialized value in the file comes from a `mayf-fixed-drivers` type:
//! the bytes from its `to_le_bytes`, and each rounded value from its
//! `checked_from_ratio_round`.
//!
//! The decimal text of a value comes from long division on Dashu integers.
//! The TypeScript package converts by multiplication instead, so the two
//! sides share no method.

use std::collections::BTreeSet;
use std::num::NonZeroU128;

use dashu::base::{DivRem, Sign};
use dashu::integer::{IBig, UBig};
use mayf_fixed_drivers::{I64F64, I64Scaled, U64F64, U128F128, U192F64, U256Scaled};
use mayf_fixed_number::{DirectedRounding, FixedNumber, MathResult};

/// The fixture file inside the TypeScript package.
const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../ts/fixed-number-ts/test/fixtures/mayf-fixed-drivers.json"
);

/// What the fixtures need from one driver format.
trait Limb: Copy {
    fn signed() -> bool;
    fn one() -> Self;
    fn min() -> Self;
    fn max() -> Self;
    fn from_le(bytes: &[u8]) -> Self;
    fn to_le(self) -> Vec<u8>;
    fn from_ratio(
        numerator: i128,
        denominator: NonZeroU128,
        rounding: DirectedRounding,
    ) -> MathResult<Self>;

    /// The raw scaled integer: the storage bytes read as one number. A signed
    /// driver stores two's complement, as a Rust integer does.
    fn raw(self) -> IBig {
        let bytes = self.to_le();
        if Self::signed() {
            IBig::from_le_bytes(&bytes)
        } else {
            UBig::from_le_bytes(&bytes).into()
        }
    }
}

macro_rules! limb {
    ($driver:ty $(, $($generics:tt)+)?) => {
        impl$(<$($generics)+>)? Limb for $driver {
            fn signed() -> bool {
                Self::MIN != Self::ZERO
            }

            fn one() -> Self {
                Self::ONE
            }

            fn min() -> Self {
                Self::MIN
            }

            fn max() -> Self {
                Self::MAX
            }

            fn from_le(bytes: &[u8]) -> Self {
                Self::from_le_bytes(bytes.try_into().expect("the driver's storage width"))
            }

            fn to_le(self) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }

            fn from_ratio(
                numerator: i128,
                denominator: NonZeroU128,
                rounding: DirectedRounding,
            ) -> MathResult<Self> {
                Self::checked_from_ratio_round(numerator, denominator, rounding)
            }
        }
    };
}

limb!(I64F64);
limb!(U64F64);
limb!(U192F64);
limb!(U128F128);
limb!(I64Scaled<SCALE>, const SCALE: u64);
limb!(U256Scaled<SCALE>, const SCALE: u64);

/// SplitMix64. The fixtures need a fixed sequence, not good randomness.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn bytes(&mut self, count: usize) -> Vec<u8> {
        (0..count).map(|_| self.next() as u8).collect()
    }
}

/// The exact decimal text of `numerator / denominator`, with no exponent and
/// no trailing zeros.
///
/// Long division: each step multiplies the remainder by ten and takes one
/// digit. Every denominator here divides a power of ten, so the remainder
/// reaches zero.
fn decimal_text(numerator: &IBig, denominator: &UBig) -> String {
    let (sign, magnitude) = numerator.clone().into_parts();
    let (whole, mut remainder) = magnitude.div_rem(denominator);
    let mut text = whole.to_string();
    if remainder != UBig::ZERO {
        text.push('.');
    }
    while remainder != UBig::ZERO {
        let (digit, rest) = (remainder * UBig::from(10u8)).div_rem(denominator);
        text += &digit.to_string();
        remainder = rest;
        assert!(text.len() < 1_000, "the decimal expansion does not end");
    }
    if sign == Sign::Negative {
        text.insert(0, '-');
    }
    text
}

/// Storage byte patterns for one driver. Every pattern of the right width is a
/// valid value of every driver.
fn storage_patterns<L: Limb>() -> Vec<Vec<u8>> {
    let width = L::one().to_le().len();
    let mut patterns = vec![vec![0x00; width], vec![0xff; width]];
    patterns.extend([L::one(), L::min(), L::max()].map(L::to_le));

    // The three bits around each 64-bit limb edge: the bit alone, then every
    // bit below it. The top bit is the sign bit of a signed driver.
    for bit in (0..width * 8).filter(|bit| matches!(bit % 64, 0 | 1 | 63)) {
        let mut alone = vec![0; width];
        alone[bit / 8] = 1 << (bit % 8);
        let mut below = vec![0; width];
        below[..bit / 8].fill(0xff);
        below[bit / 8] = (1 << (bit % 8)) - 1;
        patterns.extend([alone, below]);
    }

    let mut random = SplitMix64(0);
    for _ in 0..16 {
        patterns.push(random.bytes(width));
    }
    // Random low bytes under zeros give small values. Under ones they give
    // small negative values, or large unsigned ones.
    for fill in [0x00, 0xff] {
        for _ in 0..8 {
            let mut pattern = vec![fill; width];
            let count = 1 + random.below(width - 1);
            pattern[..count].copy_from_slice(&random.bytes(count));
            patterns.push(pattern);
        }
    }

    let mut seen = BTreeSet::new();
    patterns.retain(|pattern| seen.insert(pattern.clone()));
    patterns
}

/// Decimal numbers as `(numerator, decimals)`: the number is
/// `numerator / 10^decimals`. Every driver rounds the same list.
fn ratio_cases() -> Vec<(i128, u32)> {
    let i64_max = i128::from(i64::MAX);
    let i64_min = i128::from(i64::MIN);
    let u64_max = i128::from(u64::MAX);
    let mut cases = vec![
        (0, 0),
        (1, 0),
        (-1, 0),
        // On every binary grid.
        (5, 1),
        (-5, 1),
        (125, 3),
        // Off the binary grids.
        (1, 1),
        (-1, 1),
        (234_123, 3),
        (-234_123, 3),
        // One quantum of the 1e-12 grid, then one and a half.
        (1, 12),
        (15, 13),
        (-15, 13),
        // Below one quantum of the 2^-64 grids.
        (1, 38),
        (-1, 38),
        // The ends of I64F64: [-2^63, 2^63).
        (i64_max, 0),
        (i64_max + 1, 0),
        (i64_max * 10 + 9, 1),
        (i64_min, 0),
        (i64_min * 10 - 1, 1),
        // The upper end of U64F64: [0, 2^64).
        (u64_max, 0),
        (u64_max + 1, 0),
        (u64_max * 10 + 9, 1),
        // The ends of I64Scaled<10^12>, then half a quantum beyond each.
        (i64_max, 12),
        (i64_max * 10 + 5, 13),
        (i64_min, 12),
        (i64_min * 10 - 5, 13),
        // The same for I64Scaled<8>. One eighth is 125 thousandths.
        (i64_max * 125, 3),
        (i64_max * 1_250 + 5, 4),
        (i64_min * 125, 3),
        (i64_min * 1_250 - 5, 4),
        // The widest inputs the drivers accept.
        (i128::MAX, 0),
        (i128::MIN, 0),
        (i128::MAX, 38),
        (i128::MIN, 38),
    ];

    let mut random = SplitMix64(1);
    for _ in 0..12 {
        let wide = (u128::from(random.next()) << 64) | u128::from(random.next());
        let magnitude = (wide >> (1 + random.below(127))) as i128;
        let sign = if random.below(2) == 0 { 1 } else { -1 };
        cases.push((sign * magnitude, random.below(39) as u32));
    }
    cases
}

/// The JSON member for one driver.
fn member<L: Limb>(label: &str) -> String {
    let (_, one) = L::one().raw().into_parts();

    let values: Vec<String> = storage_patterns::<L>()
        .iter()
        .map(|bytes| {
            let value = L::from_le(bytes);
            let hex: String = value
                .to_le()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            let raw = value.raw();
            let decimal = decimal_text(&raw, &one);
            format!(r#"      {{ "leBytes": "{hex}", "raw": "{raw}", "decimal": "{decimal}" }}"#)
        })
        .collect();

    let ratios: Vec<String> = ratio_cases()
        .into_iter()
        .map(|(numerator, decimals)| {
            let denominator =
                NonZeroU128::new(10u128.pow(decimals)).expect("a power of ten is not zero");
            let rounded = |rounding| match L::from_ratio(numerator, denominator, rounding) {
                Ok(value) => format!(r#"{{ "ok": "{}" }}"#, value.raw()),
                Err(error) => format!(r#"{{ "err": "{error:?}" }}"#),
            };
            let decimal = decimal_text(&numerator.into(), &denominator.get().into());
            let floor = rounded(DirectedRounding::Floor);
            let ceil = rounded(DirectedRounding::Ceil);
            format!(r#"      {{ "decimal": "{decimal}", "floor": {floor}, "ceil": {ceil} }}"#)
        })
        .collect();

    let signed = L::signed();
    let bytes = L::one().to_le().len();
    let min = L::min().raw();
    let max = L::max().raw();
    let values = values.join(",\n");
    let ratios = ratios.join(",\n");
    format!(
        r#"  "{label}": {{
    "signed": {signed},
    "bytes": {bytes},
    "one": "{one}",
    "min": "{min}",
    "max": "{max}",
    "values": [
{values}
    ],
    "ratios": [
{ratios}
    ]
  }}"#
    )
}

/// The whole fixture file. Every string in it is digits, hex, a sign, a point
/// or an identifier, so none needs JSON escaping.
fn fixtures() -> String {
    let members = [
        member::<I64F64>("I64F64"),
        member::<U64F64>("U64F64"),
        member::<U192F64>("U192F64"),
        member::<U128F128>("U128F128"),
        // 10^12 is the pico grid. 8 has only twos and 250 has more fives than
        // twos, so each needs a different count of decimal places.
        member::<I64Scaled<1_000_000_000_000>>("I64Scaled<1000000000000>"),
        member::<I64Scaled<8>>("I64Scaled<8>"),
        member::<U256Scaled<1_000_000_000_000>>("U256Scaled<1000000000000>"),
        member::<U256Scaled<250>>("U256Scaled<250>"),
    ];
    format!("{{\n{}\n}}\n", members.join(",\n"))
}

fn main() -> std::io::Result<()> {
    std::fs::write(FIXTURES, fixtures())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-checked expansions. The fixtures are only as good as this oracle.
    #[test]
    fn long_division_gives_exact_decimal_text() {
        let q64 = UBig::ONE << 64;
        assert_eq!(decimal_text(&IBig::ZERO, &q64), "0");
        assert_eq!(decimal_text(&-(IBig::from(3) << 63), &q64), "-1.5");
        // 2^-64 is 5^64 / 10^64, and 5^64 has 45 digits.
        assert_eq!(
            decimal_text(&IBig::ONE, &q64),
            "0.0000000000000000000542101086242752217003726400434970855712890625"
        );
        assert_eq!(
            decimal_text(&IBig::from(1_200), &UBig::from(1_000u16)),
            "1.2"
        );
    }

    /// The TypeScript tests read the checked-in file, so it must hold what the
    /// drivers produce today.
    #[test]
    fn checked_in_fixtures_match_the_drivers() {
        let checked_in = std::fs::read_to_string(FIXTURES).expect("the checked-in fixtures");
        assert!(
            checked_in == fixtures(),
            "{FIXTURES} is stale; run `cargo run -p mayf-fixed-drivers-fixture-gen`"
        );
    }
}

//! Numeric mechanics independent of any production kernel or curve formula.
//! A coarse decimal grid and exact integer oracles expose endpoint/rounding
//! mistakes that high-precision domain examples can otherwise conceal.
use core::num::NonZeroU128;
use mayf_fixed_number::driver::{Floored, Seal};
use mayf_fixed_number::{
    Bounds, DirectedRounding, FixedNumber, MathError, MathResult, NonNeg, Pos, SignedFixed,
    certified,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Tenths(i16);

fn rounded(n: i128, d: i128, direction: DirectedRounding) -> MathResult<Tenths> {
    if d == 0 {
        return Err(MathError::DivisionByZero);
    }
    let (n, d) = if d < 0 { (-n, -d) } else { (n, d) };
    let floor = n.div_euclid(d);
    let raw = floor + i128::from(direction == DirectedRounding::Ceil && n.rem_euclid(d) != 0);
    Ok(Tenths(raw.try_into().map_err(|_| MathError::Overflow)?))
}

impl FixedNumber for Tenths {
    const ZERO: Self = Self(0);
    const ONE: Self = Self(10);
    const QUANTUM: Self = Self(1);
    fn checked_from_i128(n: i128) -> MathResult<Self> {
        let raw = n.checked_mul(10).ok_or(MathError::Overflow)?;
        Ok(Self(raw.try_into().map_err(|_| MathError::Overflow)?))
    }
    fn checked_add(self, rhs: Self) -> MathResult<Self> {
        self.0
            .checked_add(rhs.0)
            .map(Self)
            .ok_or(MathError::Overflow)
    }
    fn checked_sub(self, rhs: Self) -> MathResult<Self> {
        self.0
            .checked_sub(rhs.0)
            .map(Self)
            .ok_or(MathError::Overflow)
    }
    fn mul_div_floor(self, mul: Self, div: Self, _: Seal) -> MathResult<Floored<Self>> {
        if self < Self::ZERO || mul < Self::ZERO || div < Self::ZERO {
            return Err(MathError::Domain);
        }
        let value = self.checked_mul_div_floor(mul, div)?;
        Ok(Floored {
            value,
            inexact: (i32::from(self.0) * i32::from(mul.0)) % i32::from(div.0) != 0,
        })
    }
    fn ratio_floor(n: u128, d: NonZeroU128, _: Seal) -> MathResult<Floored<Self>> {
        let n = n.checked_mul(10).ok_or(MathError::UnsupportedRange)?;
        Ok(Floored {
            value: Self((n / d.get()).try_into().map_err(|_| MathError::Overflow)?),
            inexact: n % d.get() != 0,
        })
    }
    fn checked_mul_round(self, rhs: Self, direction: DirectedRounding) -> MathResult<Self> {
        rounded(i128::from(self.0) * i128::from(rhs.0), 10, direction)
    }
    fn checked_div_round(self, rhs: Self, direction: DirectedRounding) -> MathResult<Self> {
        rounded(i128::from(self.0) * 10, i128::from(rhs.0), direction)
    }
    fn checked_mul_div_round(
        self,
        mul: Self,
        div: Self,
        direction: DirectedRounding,
    ) -> MathResult<Self> {
        rounded(
            i128::from(self.0) * i128::from(mul.0),
            i128::from(div.0),
            direction,
        )
    }
}
impl SignedFixed for Tenths {
    fn checked_neg(self) -> MathResult<Self> {
        self.0.checked_neg().map(Self).ok_or(MathError::Overflow)
    }
}
fn bounds(lo: i16, hi: i16) -> Bounds<Tenths> {
    Bounds::new(Tenths(lo), Tenths(hi)).unwrap()
}
fn raw(b: Bounds<Tenths>) -> (i16, i16) {
    (b.lower().0, b.upper().0)
}
fn positive(raw: i16) -> Pos<Tenths> {
    Pos::new(Tenths(raw)).unwrap()
}

#[test]
fn ordered_bounds_are_established_at_construction() {
    for lo in -20..=20 {
        for hi in -20..=20 {
            let actual = Bounds::new(Tenths(lo), Tenths(hi));
            if lo > hi {
                assert_eq!(actual, Err(MathError::Domain));
            } else {
                let actual = actual.unwrap();
                assert_eq!(raw(actual), (lo, hi));
                assert_eq!(actual.is_exact(), lo == hi);
            }
        }
    }
    assert_eq!(raw(Bounds::exact(Tenths(-7))), (-7, -7));
    assert_eq!(
        raw(bounds(-4, 8).intersection(bounds(2, 12)).unwrap()),
        (2, 8)
    );
    assert_eq!(
        bounds(0, 1).intersection(bounds(2, 3)),
        Err(MathError::Domain)
    );
}

#[test]
fn signed_sums_differences_and_negation_match_exact_endpoint_oracles() {
    let endpoints = [-12_i16, -7, -1, 0, 1, 7, 12];
    for a in endpoints {
        for b in endpoints.into_iter().filter(|b| *b >= a) {
            for c in endpoints {
                for d in endpoints.into_iter().filter(|d| *d >= c) {
                    let left = bounds(a, b);
                    let right = bounds(c, d);
                    assert_eq!(raw(left.checked_add(right).unwrap()), (a + c, b + d));
                    assert_eq!(raw(left.checked_sub(right).unwrap()), (a - d, b - c));
                    assert_eq!(raw(left.checked_neg().unwrap()), (-b, -a));
                }
            }
        }
    }
}

#[test]
fn signed_point_products_enclose_the_exact_product() {
    for a in -20_i16..=20 {
        for b in -20_i16..=20 {
            let product = i32::from(a) * i32::from(b);
            let expected = (product.div_euclid(10), -(-product).div_euclid(10));
            assert_eq!(
                raw(Bounds::product(Tenths(a), Tenths(b)).unwrap()),
                (expected.0 as i16, expected.1 as i16)
            );
        }
    }
    assert_eq!(
        Bounds::product(Tenths(i16::MIN), Tenths(-10)),
        Err(MathError::Overflow)
    );
}

#[test]
fn exact_operand_scaling_fuses_and_reverses_negative_endpoints() {
    for lo in -12..=12 {
        for hi in lo..=12 {
            for multiplier in [-17_i16, -1, 0, 1, 17] {
                for divisor in [1_i16, 3, 10, 29] {
                    let actual = bounds(lo, hi)
                        .checked_mul_div(Tenths(multiplier), positive(divisor))
                        .unwrap();
                    let corners = [
                        i32::from(lo) * i32::from(multiplier),
                        i32::from(hi) * i32::from(multiplier),
                    ];
                    let floor = corners
                        .into_iter()
                        .min()
                        .unwrap()
                        .div_euclid(i32::from(divisor));
                    let ceil =
                        -(-corners.into_iter().max().unwrap()).div_euclid(i32::from(divisor));
                    assert_eq!(raw(actual), (floor as i16, ceil as i16));
                }
            }
        }
    }
    // The product exceeds the grid's storage while the fused quotient fits.
    assert_eq!(
        Bounds::product(Tenths(32_000), Tenths(20)),
        Err(MathError::Overflow)
    );
    assert_eq!(
        raw(bounds(30_000, 32_000)
            .checked_mul_div(Tenths(20), positive(20))
            .unwrap()),
        (30_000, 32_000)
    );
}

#[test]
fn required_error_allowances_and_negation_never_clip_overflow() {
    let quantum = NonNeg::new(Tenths::QUANTUM).unwrap();
    assert_eq!(raw(bounds(-3, 5).checked_widen(quantum).unwrap()), (-4, 6));
    assert_eq!(
        bounds(0, i16::MAX).checked_widen(quantum),
        Err(MathError::Overflow)
    );
    assert_eq!(
        bounds(i16::MIN, 0).checked_widen(quantum),
        Err(MathError::Overflow)
    );
    assert_eq!(bounds(i16::MIN, 0).checked_neg(), Err(MathError::Overflow));
}

#[test]
fn a_sign_certificate_is_distinct_from_checking_the_endpoints() {
    let uncertain = bounds(-1, 3);
    assert_eq!(uncertain.try_nonnegative(), Err(MathError::Domain));
    assert_eq!(
        raw(certified::nonnegative(uncertain)
            .bounds()
            .unwrap()
            .into_raw()),
        (0, 3)
    );
    assert_eq!(
        certified::nonnegative(bounds(-3, -1)).bounds(),
        Err(MathError::Domain)
    );
    let known = bounds(2, 7).try_nonnegative().unwrap();
    assert_eq!(known.bounded().bounds(), Ok(known));
    assert_eq!(known.into_raw(), bounds(2, 7));
}

#[test]
fn expression_bridges_keep_fusion_and_explicit_materialization() {
    let x = bounds(30_000, 32_000).try_nonnegative().unwrap();
    let two = positive(20);
    assert_eq!((x.bounded() * two / two).bounds(), Ok(x));
    assert_eq!(
        (x.bounded() * two).materialize().bounds(),
        Err(MathError::Overflow)
    );
    let quantum = positive(1);
    assert_eq!(
        raw((quantum.bounded() * quantum / quantum)
            .bounds()
            .unwrap()
            .into_raw()),
        (1, 1)
    );
    assert_eq!(
        raw(((quantum.bounded() * quantum).materialize() / quantum)
            .bounds()
            .unwrap()
            .into_raw()),
        (0, 10)
    );
}

#[test]
fn refined_points_use_the_nonnegative_arithmetic_path() {
    let quantum = positive(1);
    let two = positive(20);
    let zero = NonNeg::zero();
    let nonnegative: NonNeg<Tenths> = quantum.into();
    assert_eq!(nonnegative.checked_mul_floor(quantum), Ok(zero));
    assert_eq!(nonnegative.checked_mul_ceil(quantum), Ok(nonnegative));
    assert_eq!(nonnegative.checked_div_floor(two), Ok(zero));
    assert_eq!(nonnegative.checked_div_ceil(two), Ok(nonnegative));
    assert_eq!(nonnegative.checked_mul_div_floor(quantum, two), Ok(zero));
    assert_eq!(
        nonnegative.checked_mul_div_ceil(quantum, two),
        Ok(nonnegative)
    );
    assert_eq!(Pos::try_from(zero), Err(MathError::Domain));
    assert_eq!(zero.checked_add(quantum), Ok(nonnegative));
    assert_eq!(NonNeg::<Tenths>::from_u64(4), NonNeg::new(Tenths(40)));
    assert_eq!(
        NonNeg::<Tenths>::from_u64(u64::MAX),
        Err(MathError::Overflow)
    );
}

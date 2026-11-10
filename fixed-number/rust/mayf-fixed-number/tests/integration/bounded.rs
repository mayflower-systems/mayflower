//! Independent coarse-grid checks: a custom driver implements only the new
//! primitives. Small exhaustive grids make direction mistakes visible.
use core::num::NonZeroU128;
use mayf_fixed_number::driver::{Floored, Seal};
use mayf_fixed_number::{
    AboveOne, Bounded, Bounds, FixedNumber, MathError, MathResult, NonNeg, Pos, certified,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Small(i16);

fn floor(n: u128, d: u128) -> MathResult<Floored<Small>> {
    if d == 0 {
        return Err(MathError::DivisionByZero);
    }
    Ok(Floored {
        value: Small((n / d).try_into().map_err(|_| MathError::Overflow)?),
        inexact: !n.is_multiple_of(d),
    })
}

impl FixedNumber for Small {
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
        if self.0 < 0 || mul.0 < 0 || div.0 < 0 {
            return Err(MathError::Domain);
        }
        floor(self.0 as u128 * mul.0 as u128, div.0 as u128)
    }
    fn ratio_floor(n: u128, d: NonZeroU128, _: Seal) -> MathResult<Floored<Self>> {
        floor(n.checked_mul(10).ok_or(MathError::Overflow)?, d.get())
    }
}
type B = Bounded<Small>;
fn ratio(n: u64, d: u64) -> B {
    B::from(n) / d
}
fn interval(lo: i16, hi: i16) -> B {
    Bounds::new(Small(lo), Small(hi))
        .unwrap()
        .try_nonnegative()
        .unwrap()
        .bounded()
}
fn edges(value: B) -> MathResult<(i16, i16)> {
    value
        .bounds()
        .map(|b| (b.lower().get().0, b.upper().get().0))
}
fn encloses(value: B, n: i128, d: i128) {
    let (lo, hi) = edges(value).unwrap();
    assert!(lo >= 0 && lo <= hi);
    assert!(i128::from(lo) * d <= n * 10, "{lo} > 10*{n}/{d}");
    assert!(i128::from(hi) * d >= n * 10, "{hi} < 10*{n}/{d}");
}

#[test]
fn syntax_retains_bounds_through_helpers_and_refined_operands() {
    fn helper<N: FixedNumber>(x: Bounded<N>, y: NonNeg<N>) -> Bounded<N> {
        y * x / 3 + 1
    }
    let x = ratio(1, 3);
    let n = NonNeg::new(Small(20)).unwrap();
    let p = Pos::new(Small(20)).unwrap();
    let a = AboveOne::new(Small(20)).unwrap();
    encloses(helper(x, n), 11, 9);
    for value in [
        n * x,
        x * n,
        p * x,
        x * p,
        a * x,
        x * a,
        x * Small(20),
        2 * x,
        x * 2_u64,
        x * 2_u32,
    ] {
        encloses(value, 2, 3);
    }
    encloses(1 + x, 4, 3);
    encloses(1 - x, 2, 3);
    encloses(1 / x, 3, 1);
    assert_eq!(edges(B::from(-1)), Err(MathError::Domain));
    assert_eq!(NonNeg::new(Small(-1)), Err(MathError::Domain));
    assert_eq!(Pos::new(Small(0)), Err(MathError::Domain));
    assert_eq!(edges(B::exact(Small(-1))), Err(MathError::Domain));
    assert_eq!(edges(x / Small(0)), Err(MathError::DivisionByZero));
}

#[test]
fn full_enclosure_automatically_reverses_fee_and_divisor_directions() {
    let payout = B::from(100) * (1 - ratio(1, 3));
    assert_eq!(edges(payout), Ok((600, 700)));
    encloses(payout, 200, 3);
    assert_eq!(edges(B::from(1) / ratio(1, 3)), Ok((25, 34)));
    assert_eq!(edges(ratio(1, 3) * 3), Ok((9, 12)));
}

#[test]
fn fusion_survives_helpers_and_observations_but_materialization_is_explicit() {
    fn identity<N: FixedNumber>(x: Bounded<N>) -> Bounded<N> {
        x
    }
    let max = Small(i16::MAX).bounded();
    let product = identity(max * 2);
    assert_eq!(product.bounds(), Err(MathError::Overflow));
    // The failed observation evaluated a copy; fusion still rescues the range.
    assert_eq!(edges(product / 2), Ok((i16::MAX, i16::MAX)));
    assert_eq!(edges(product.materialize() / 2), Err(MathError::Overflow));
    let tiny_product = Small(1).bounded() * Small(1);
    assert_eq!(edges(tiny_product / Small(1)), Ok((1, 1)));
    assert_eq!(edges(tiny_product.materialize() / Small(1)), Ok((0, 10)));
    assert_eq!(edges((tiny_product + 0) / Small(1)), Ok((0, 10)));
}

#[test]
fn subtraction_contracts_and_certificates_are_distinct() {
    let a = interval(9, 11);
    let b = interval(10, 12);
    assert_eq!(edges(a - b), Err(MathError::Domain));
    // Certified exact values can be ordered even when their enclosures overlap.
    assert_eq!(edges(certified::sub_nonnegative(a, b)), Ok((0, 1)));
    assert_eq!(
        edges(certified::sub_nonnegative(interval(0, 1), interval(2, 3))),
        Err(MathError::Domain)
    );
}

#[test]
fn errors_never_disappear_at_identities_or_materialization() {
    let zero = B::from(0);
    let bad = B::from(1) / zero;
    for value in [
        bad + zero,
        zero + bad,
        bad - zero,
        zero - bad,
        bad * zero,
        zero * bad,
        bad / zero,
        zero / bad,
        bad.materialize(),
        certified::sub_nonnegative(zero, bad),
    ] {
        assert_eq!(edges(value), Err(MathError::DivisionByZero));
    }
    let overflow = (Small(i16::MAX).bounded() * 2).materialize();
    assert_eq!(edges(bad + overflow), Err(MathError::DivisionByZero));
    assert_eq!(edges(overflow + bad), Err(MathError::Overflow));
    assert_eq!(edges(zero / zero), Err(MathError::DivisionByZero));
    assert_eq!(edges(zero / interval(0, 1)), Err(MathError::Domain));
    // A representable lower endpoint never hides an overflowing upper one.
    let just_above_max = Small(i16::MAX).bounded() + Small(1).bounded() / 10;
    assert_eq!(edges(just_above_max), Err(MathError::Overflow));
}

#[test]
fn exhaustive_small_intervals_enclose_all_operand_corner_values() {
    // Every possible interval on this grid; exact values at corners are an
    // independent rational oracle, including dependency-free widest cases.
    for al in 0_i16..=8 {
        for au in al..=8 {
            for bl in 0_i16..=8 {
                for bu in bl..=8 {
                    let a = interval(al, au);
                    let b = interval(bl, bu);
                    for x in [al, au] {
                        for y in [bl, bu] {
                            let x = i128::from(x);
                            let y = i128::from(y);
                            encloses(a + b, x + y, 10);
                            encloses(a * b, x * y, 100);
                            if al >= bu {
                                encloses(a - b, x - y, 10);
                            }
                            if bl > 0 {
                                encloses(a / b, x, y);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Rational {
    n: i128,
    d: i128,
}
impl Rational {
    fn new(n: i128, d: i128) -> Self {
        let (n, d) = if d < 0 { (-n, -d) } else { (n, d) };
        let (mut a, mut b) = (n.abs(), d);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        Self { n: n / a, d: d / a }
    }
}
fn next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state >> 32
}
fn tree(state: &mut u64, depth: u32, successes: &mut usize) -> (B, Rational) {
    if depth == 0 {
        let raw = (next(state) % 21) as i16;
        return (Small(raw).bounded(), Rational::new(i128::from(raw), 10));
    }
    let (a, x) = tree(state, depth - 1, successes);
    let (b, y) = if next(state).is_multiple_of(5) {
        (a, x)
    } else {
        tree(state, depth - 1, successes)
    };
    let (value, exact) = match next(state) % 4 {
        0 => (a + b, Rational::new(x.n * y.d + y.n * x.d, x.d * y.d)),
        1 => (a - b, Rational::new(x.n * y.d - y.n * x.d, x.d * y.d)),
        2 => (a * b, Rational::new(x.n * y.n, x.d * y.d)),
        _ if y.n != 0 => (a / b, Rational::new(x.n * y.d, x.d * y.n)),
        _ => (a + b, Rational::new(x.n * y.d + y.n * x.d, x.d * y.d)),
    };
    if value.bounds().is_ok() {
        encloses(value, exact.n, exact.d);
        *successes += 1;
    }
    (value, exact)
}

#[test]
fn generated_chains_preserve_containment_at_every_successful_node() {
    let mut seed = 0x9ddfea08eb382d69;
    let mut successes = 0;
    for _ in 0..10_000 {
        let _ = tree(&mut seed, 3, &mut successes);
    }
    assert!(
        successes > 30_000,
        "the supported grid must actually evaluate useful expressions"
    );
}

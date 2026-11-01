//! High-signal property and fuzz tests for `mayf-characteristic` using `mayf-fixed-dashu` (`DashuFixed12`).
//!
//! Exercises:
//! - Complete error taxonomy fuzzing on `AffineAnchor` and `AffineTransform`.
//! - Identity transform invariants and sensitivity fuzzing.
//! - Anchored affine continuous evaluation: boundary continuity at $x = anchor\_x$,
//!   monotonicity, and domain preservation ($x \ge anchor\_x$).
//! - Integration and upward area solving across affine transforms with high-precision DashuFixed12 arithmetic.

use mayf_characteristic::{AffineAnchor, AffineTransform, AffineTransformError, Characteristic};
use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_number::{Bounds, FixedNumber, NonNeg, Pos};
use mayf_monotone::{MonotoneCalculator, MonotoneCalculatorError, MonotoneCalculatorResult};

type Fx = DashuFixed12;

const RAW_ONE: i128 = 1_000_000_000_000;

fn fx_raw(raw: i128) -> Fx {
    Fx::from_scaled_i128(raw)
}

fn nn_raw(raw: i128) -> NonNeg<Fx> {
    NonNeg::new(fx_raw(raw)).unwrap()
}

fn n(val: i64) -> Fx {
    Fx::checked_from_i128(i128::from(val)).unwrap()
}

fn nn(val: i64) -> NonNeg<Fx> {
    NonNeg::new(n(val)).unwrap()
}

#[derive(Clone, Copy)]
struct Fuzzer(u128);

impl Fuzzer {
    fn new(seed: u128) -> Self {
        Self(seed)
    }

    fn next_u128(&mut self) -> u128 {
        self.0 = self
            .0
            .wrapping_mul(0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645)
            .wrapping_add(0x1405_7b7e_f767_814f);
        self.0
    }

    fn signed_raw(&mut self, max_whole: i128) -> i128 {
        let max = max_whole * RAW_ONE;
        let mag = (self.next_u128() % (max as u128 + 1)) as i128;
        if (self.next_u128() & 1) == 0 {
            mag
        } else {
            -mag
        }
    }
}

// ---------------------------------------------------------------------------
// 1. AffineAnchor & AffineTransform Construction Fuzzing
// ---------------------------------------------------------------------------

#[test]
fn fuzz_affine_anchor_validation() {
    let mut fuzz = Fuzzer::new(0xface_b00c_cafe_feed_1357_9bdf_0246_8ace);

    for _ in 0..1000 {
        let x_raw = fuzz.signed_raw(50);
        let y_lo_raw = fuzz.signed_raw(50);
        let y_hi_raw = fuzz.signed_raw(50);

        let Ok(y) = Bounds::new(fx_raw(y_lo_raw), fx_raw(y_hi_raw)) else {
            // Inverted intervals cannot cross the numeric boundary.
            continue;
        };
        let anchor_res = AffineAnchor::try_new(fx_raw(x_raw), y);

        if x_raw < 0 {
            assert_eq!(anchor_res, Err(AffineTransformError::NegativeAnchorInput));
        } else if y_lo_raw < 0 || y_hi_raw < 0 {
            assert_eq!(anchor_res, Err(AffineTransformError::NegativeAnchorOutput));
        } else {
            let anchor = anchor_res.unwrap();
            assert_eq!(anchor.x().get().to_scaled_i128(), x_raw);
            assert_eq!(anchor.y().lower().get().to_scaled_i128(), y_lo_raw);
            assert_eq!(anchor.y().upper().get().to_scaled_i128(), y_hi_raw);
        }
    }
}

#[test]
fn fuzz_affine_transform_validation() {
    let mut fuzz = Fuzzer::new(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210);

    for _ in 0..1000 {
        let x_scale_raw = fuzz.signed_raw(20);
        let x_trans_raw = fuzz.signed_raw(20);
        let y_scale_raw = fuzz.signed_raw(20);
        let anchor_x_raw = fuzz.signed_raw(20);
        let anchor_y_lo_raw = fuzz.signed_raw(20);
        let anchor_y_hi_raw = fuzz.signed_raw(20);

        let Ok(anchor_y) = Bounds::new(fx_raw(anchor_y_lo_raw), fx_raw(anchor_y_hi_raw)) else {
            continue;
        };
        let transform_res = AffineTransform::try_new(
            fx_raw(x_scale_raw),
            fx_raw(x_trans_raw),
            fx_raw(y_scale_raw),
            fx_raw(anchor_x_raw),
            anchor_y,
        );

        if x_scale_raw <= 0 {
            assert_eq!(
                transform_res,
                Err(AffineTransformError::NonPositiveHorizontalScale)
            );
        } else if y_scale_raw < 0 {
            assert_eq!(
                transform_res,
                Err(AffineTransformError::NegativeVerticalScale)
            );
        } else if anchor_x_raw < 0 {
            assert_eq!(
                transform_res,
                Err(AffineTransformError::NegativeAnchorInput)
            );
        } else if anchor_y_lo_raw < 0 || anchor_y_hi_raw < 0 {
            assert_eq!(
                transform_res,
                Err(AffineTransformError::NegativeAnchorOutput)
            );
        } else {
            let transform = transform_res.unwrap();
            assert_eq!(transform.x_scale().get().to_scaled_i128(), x_scale_raw);
            assert_eq!(transform.x_translation().to_scaled_i128(), x_trans_raw);
            assert_eq!(transform.y_scale().get().to_scaled_i128(), y_scale_raw);
            assert_eq!(transform.anchor().x().get().to_scaled_i128(), anchor_x_raw);
            assert_eq!(
                transform.anchor().y().lower().get().to_scaled_i128(),
                anchor_y_lo_raw
            );
            assert_eq!(
                transform.anchor().y().upper().get().to_scaled_i128(),
                anchor_y_hi_raw
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Identity Transform Invariants
// ---------------------------------------------------------------------------

#[test]
fn identity_transform_invariants() {
    let id = AffineTransform::<Fx>::identity();
    assert!(id.is_identity());
    assert_eq!(id.x_scale(), Pos::one());
    assert_eq!(id.x_translation(), Fx::ZERO);
    assert_eq!(id.y_scale(), nn(1));
    assert_eq!(id.anchor().x(), NonNeg::zero());
    assert_eq!(id.anchor().y().lower(), NonNeg::zero());
    assert_eq!(id.anchor().y().upper(), NonNeg::zero());

    // Perturbing any single component breaks identity:
    // 1. x_scale
    let t1 = AffineTransform::try_new(n(2), n(0), n(1), n(0), Bounds::exact(n(0))).unwrap();
    assert!(!t1.is_identity());

    // 2. x_translation
    let t2 = AffineTransform::try_new(n(1), n(1), n(1), n(0), Bounds::exact(n(0))).unwrap();
    assert!(!t2.is_identity());

    // 3. y_scale
    let t3 = AffineTransform::try_new(n(1), n(0), n(2), n(0), Bounds::exact(n(0))).unwrap();
    assert!(!t3.is_identity());

    // 4. anchor x
    let t4 = AffineTransform::try_new(n(1), n(0), n(1), n(1), Bounds::exact(n(0))).unwrap();
    assert!(!t4.is_identity());

    // 5. anchor y
    let t5 = AffineTransform::try_new(n(1), n(0), n(1), n(0), Bounds::exact(n(1))).unwrap();
    assert!(!t5.is_identity());
}

// ---------------------------------------------------------------------------
// 3. Characteristic Anchored Affine Evaluation & Monotonicity
// ---------------------------------------------------------------------------

/// Concrete linear characteristic $f(u) = slope \cdot u$ implementing `Characteristic`.
#[derive(Clone, Copy)]
struct LinearShape {
    slope: NonNeg<Fx>,
}

struct AnchoredLinearCalc {
    slope: NonNeg<Fx>,
    affine: AffineTransform<Fx>,
}

impl Characteristic<Fx> for LinearShape {
    type Calculator = AnchoredLinearCalc;

    fn calculator(&self, affine: AffineTransform<Fx>) -> Self::Calculator {
        AnchoredLinearCalc {
            slope: self.slope,
            affine,
        }
    }
}

impl MonotoneCalculator<Fx> for AnchoredLinearCalc {
    fn y(&self, x: NonNeg<Fx>) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        let anchor_x = self.affine.anchor().x();
        if x < anchor_x {
            return Err(MonotoneCalculatorError::InputOutOfDomain);
        }

        // u(x) - u(anchor_x) = x_scale * (x - anchor_x)
        let delta_x = x
            .checked_sub(anchor_x)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let delta_u = delta_x
            .checked_mul_floor(NonNeg::new(self.affine.x_scale().get()).unwrap())
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        // delta_f = slope * delta_u
        let delta_f = delta_u
            .checked_mul_floor(self.slope)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        // g(x) = anchor_y + y_scale * delta_f
        let scaled_f_lo = delta_f
            .checked_mul_floor(self.affine.y_scale())
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let scaled_f_hi = delta_f
            .checked_mul_ceil(self.affine.y_scale())
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        let anchor_y = self.affine.anchor().y();
        let lo = anchor_y
            .lower()
            .checked_add(scaled_f_lo)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let hi = anchor_y
            .upper()
            .checked_add(scaled_f_hi)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        Ok(Bounds::new(lo, hi).unwrap())
    }

    fn integrate(
        &self,
        lo: NonNeg<Fx>,
        width: NonNeg<Fx>,
    ) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        let anchor_x = self.affine.anchor().x();
        if lo < anchor_x {
            return Err(MonotoneCalculatorError::InputOutOfDomain);
        }
        if width == NonNeg::zero() {
            return Ok(Bounds::exact(NonNeg::zero()));
        }

        let y_lo = self.y(lo)?;
        let hi = lo
            .checked_add(width)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let y_hi = self.y(hi)?;

        // Trapezoid bounds over [lo, hi]: width * (y(lo) + y(hi)) / 2
        let two = Pos::new(n(2)).unwrap();
        let sum_lo = y_lo
            .lower()
            .checked_add(y_hi.lower())
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let sum_hi = y_lo
            .upper()
            .checked_add(y_hi.upper())
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        let avg_lo = sum_lo
            .checked_div_floor(two)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let avg_hi = sum_hi
            .checked_div_ceil(two)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        let area_lo = width
            .checked_mul_floor(avg_lo)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let area_hi = width
            .checked_mul_ceil(avg_hi)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        Ok(Bounds::new(area_lo, area_hi).unwrap())
    }
}

#[test]
fn fuzz_anchored_affine_continuity_and_monotonicity() {
    let mut fuzz = Fuzzer::new(0x7788_99aa_bbcc_ddee_ff00_1122_3344_5566);

    for _ in 0..200 {
        let slope_raw = (fuzz.next_u128() % (10 * RAW_ONE as u128) + 1) as i128;
        let shape = LinearShape {
            slope: nn_raw(slope_raw),
        };

        let x_scale_raw = (fuzz.next_u128() % (5 * RAW_ONE as u128) + RAW_ONE as u128) as i128;
        let y_scale_raw = (fuzz.next_u128() % (5 * RAW_ONE as u128) + RAW_ONE as u128) as i128;
        let anchor_x_raw = (fuzz.next_u128() % (20 * RAW_ONE as u128) + 1) as i128;
        let anchor_y_raw = (fuzz.next_u128() % (50 * RAW_ONE as u128) + 1) as i128;

        let affine = AffineTransform::try_new(
            fx_raw(x_scale_raw),
            n(0),
            fx_raw(y_scale_raw),
            fx_raw(anchor_x_raw),
            Bounds::exact(fx_raw(anchor_y_raw)),
        )
        .unwrap();

        let calc = shape.calculator(affine);
        let anchor_x = nn_raw(anchor_x_raw);
        let anchor_y = nn_raw(anchor_y_raw);

        // 1. Boundary continuity invariant: at x = anchor_x, g(anchor_x) == anchor_y exactly!
        let at_anchor = calc.y(anchor_x).unwrap();
        assert_eq!(at_anchor.lower(), anchor_y);
        assert_eq!(at_anchor.upper(), anchor_y);

        // 2. Domain check: x < anchor_x must return InputOutOfDomain
        let below_anchor = anchor_x
            .checked_sub(NonNeg::new(Fx::QUANTUM).unwrap())
            .unwrap();
        assert_eq!(
            calc.y(below_anchor),
            Err(MonotoneCalculatorError::InputOutOfDomain)
        );

        // 3. Monotonicity check over [anchor_x, anchor_x + 30]:
        let mut prev_y = anchor_y;
        for step in 1..=20 {
            let x = anchor_x.checked_add(nn(step)).unwrap();
            let y_bounds = calc.y(x).unwrap();
            assert!(y_bounds.lower() >= prev_y);
            assert!(y_bounds.lower() <= y_bounds.upper());
            prev_y = y_bounds.lower();
        }

        // 4. Area solving roundtrip:
        let width = nn(10);
        let forward_area = calc.integrate(anchor_x, width).unwrap().upper();
        let sol = calc
            .max_hi_for_area_at_or_below(anchor_x, None, forward_area)
            .unwrap()
            .unwrap();

        let expected_hi = anchor_x.checked_add(width).unwrap();
        let diff = if sol.endpoint >= expected_hi {
            sol.endpoint.checked_sub(expected_hi).unwrap()
        } else {
            expected_hi.checked_sub(sol.endpoint).unwrap()
        };
        assert!(
            diff.get().to_scaled_i128() <= 4,
            "solved endpoint {:?} must be within 4 quanta of expected {:?}",
            sol.endpoint,
            expected_hi
        );
    }
}

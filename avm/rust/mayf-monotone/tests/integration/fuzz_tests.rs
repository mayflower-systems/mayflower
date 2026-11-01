//! High-signal property and fuzz tests for `mayf-monotone` using `mayf-fixed-dashu` (`DashuFixed12`).
//!
//! Exercises:
//! - Upward area solving (`max_hi_for_area_at_or_below`) and discrete $10^{-12}$ lattice maximality.
//! - Downward area solving (`max_lo_for_area_at_or_above`) and discrete $10^{-12}$ lattice minimality.
//! - Strict area conservation, interval constraints, and `hi_limit` clamping.
//! - Monotonicity across continuous area progressions.
//! - Boundary conditions (zero area, single quantum, unreachable areas).
//! - Signed enclosure validation (`require_nonnegative`).

use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_number::{Bounds, FixedNumber, NonNeg, Pos};
use mayf_monotone::{
    MonotoneCalculator, MonotoneCalculatorError, MonotoneCalculatorResult, require_nonnegative,
};

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

    fn nonnegative_raw(&mut self, max_whole: i128) -> i128 {
        let max = max_whole * RAW_ONE;
        match self.next_u128() % 10 {
            0 => 0,
            1 => 1,
            2 => RAW_ONE - 1,
            3 => RAW_ONE,
            4 => RAW_ONE + 1,
            5 => max / 2,
            6 => max - 1,
            7 => max,
            _ => (self.next_u128() % (max as u128 + 1)) as i128,
        }
    }
}

/// A linear monotone calculator $y(x) = slope \cdot x + intercept$
/// with exact directed-rounding integration backed by Dashu arbitrary-precision intermediates.
#[derive(Clone, Copy)]
struct LinearMonotone {
    slope: NonNeg<Fx>,
    intercept: NonNeg<Fx>,
}

impl LinearMonotone {
    fn new(slope: NonNeg<Fx>, intercept: NonNeg<Fx>) -> Self {
        Self { slope, intercept }
    }
}

impl MonotoneCalculator<Fx> for LinearMonotone {
    fn y(&self, x: NonNeg<Fx>) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        let lo = self
            .slope
            .checked_mul_floor(x)
            .and_then(|mx| mx.checked_add(self.intercept))
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let hi = self
            .slope
            .checked_mul_ceil(x)
            .and_then(|mx| mx.checked_add(self.intercept))
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        Ok(Bounds::new(lo, hi).unwrap())
    }

    fn integrate(
        &self,
        lo: NonNeg<Fx>,
        width: NonNeg<Fx>,
    ) -> MonotoneCalculatorResult<Bounds<NonNeg<Fx>>> {
        if width == NonNeg::zero() {
            return Ok(Bounds::exact(NonNeg::zero()));
        }

        // Integral of (slope * t + intercept) over [lo, lo + width]:
        // area = width * intercept + slope * width * (2 * lo + width) / 2
        let lo_base = width
            .checked_mul_floor(self.intercept)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let hi_base = width
            .checked_mul_ceil(self.intercept)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        if self.slope == NonNeg::zero() {
            return Ok(Bounds::new(lo_base, hi_base).unwrap());
        }

        let two_lo = lo
            .checked_add(lo)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let two_lo_plus_w = two_lo
            .checked_add(width)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        let two = Pos::new(n(2)).unwrap();
        let mid_x_floor = two_lo_plus_w
            .checked_div_floor(two)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let mid_x_ceil = two_lo_plus_w
            .checked_div_ceil(two)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        let slope_area_lo = self
            .slope
            .checked_mul_floor(width)
            .and_then(|sw| sw.checked_mul_floor(mid_x_floor))
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        let slope_area_hi = self
            .slope
            .checked_mul_ceil(width)
            .and_then(|sw| sw.checked_mul_ceil(mid_x_ceil))
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        let total_lo = lo_base
            .checked_add(slope_area_lo)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;
        let total_hi = hi_base
            .checked_add(slope_area_hi)
            .map_err(|_| MonotoneCalculatorError::ArithmeticFailure)?;

        Ok(Bounds::new(total_lo, total_hi).unwrap())
    }
}

// ---------------------------------------------------------------------------
// 1. Upward Area Solving Fuzz Tests & Discrete Maximality
// ---------------------------------------------------------------------------

#[test]
fn fuzz_solve_max_hi_inversion_and_lattice_maximality() {
    let mut fuzz = Fuzzer::new(0x1a2b_3c4d_5e6f_7081_92a3_b4c5_d6e7_f809);

    for _ in 0..200 {
        let slope_raw = (fuzz.next_u128() % (5 * RAW_ONE as u128) + 1) as i128;
        let intercept_raw = (fuzz.next_u128() % (10 * RAW_ONE as u128) + RAW_ONE as u128) as i128;
        let calc = LinearMonotone::new(nn_raw(slope_raw), nn_raw(intercept_raw));

        let lo_raw = fuzz.nonnegative_raw(20);
        let lo = nn_raw(lo_raw);

        let width_raw = (fuzz.next_u128() % (15 * RAW_ONE as u128) + 1) as i128;
        let width = nn_raw(width_raw);

        let expected_hi = lo.checked_add(width).unwrap();
        let area_bounds = calc.integrate(lo, width).unwrap();
        let target_area = area_bounds.upper();

        let solution = calc
            .max_hi_for_area_at_or_below(lo, None, target_area)
            .unwrap()
            .expect("must find solution for exact forward area");

        // Invariant 1: Budget Safety (achieved area <= target area)
        assert!(
            solution.area <= target_area,
            "solution area {:?} must be <= target area {:?}",
            solution.area,
            target_area
        );

        // Invariant 2: Tightness to at most 1 quantum on DashuFixed12 grid
        let diff = if solution.endpoint >= expected_hi {
            solution.endpoint.checked_sub(expected_hi).unwrap()
        } else {
            expected_hi.checked_sub(solution.endpoint).unwrap()
        };
        assert!(
            diff.get() <= Fx::QUANTUM,
            "endpoint {:?} must be within 1 quantum of expected {:?}",
            solution.endpoint,
            expected_hi
        );

        // Invariant 3: Discrete Maximality: 1 quantum beyond endpoint must exceed target_area
        let one_quantum = NonNeg::new(Fx::QUANTUM).unwrap();
        if let Ok(next_endpoint) = solution.endpoint.checked_add(one_quantum) {
            let next_width = next_endpoint.checked_sub(lo).unwrap();
            let next_area = calc.integrate(lo, next_width).unwrap().upper();
            assert!(
                next_area > target_area,
                "endpoint + QUANTUM must strictly exceed target area"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Downward Area Solving Fuzz Tests & Discrete Minimality
// ---------------------------------------------------------------------------

#[test]
fn fuzz_solve_max_lo_inversion_and_lattice_minimality() {
    let mut fuzz = Fuzzer::new(0x9876_5432_10fe_dcba_a1b2_c3d4_e5f6_0718);

    for _ in 0..200 {
        let slope_raw = (fuzz.next_u128() % (5 * RAW_ONE as u128) + 1) as i128;
        let intercept_raw = (fuzz.next_u128() % (10 * RAW_ONE as u128) + RAW_ONE as u128) as i128;
        let calc = LinearMonotone::new(nn_raw(slope_raw), nn_raw(intercept_raw));

        let lo_raw = (fuzz.next_u128() % (10 * RAW_ONE as u128) + RAW_ONE as u128) as i128;
        let lo = nn_raw(lo_raw);

        let width_raw = (fuzz.next_u128() % (15 * RAW_ONE as u128) + 1) as i128;
        let width = nn_raw(width_raw);

        let hi = lo.checked_add(width).unwrap();
        let area_bounds = calc.integrate(lo, width).unwrap();
        let target_area = area_bounds.lower();

        // Solve downward with lo_limit = 0 <= lo
        let lo_limit = NonNeg::zero();
        let solution = calc
            .max_lo_for_area_at_or_above(lo_limit, hi, target_area)
            .unwrap()
            .expect("must find solution for exact forward area");

        // Invariant 1: Target Coverage (achieved area >= target area)
        assert!(
            solution.area >= target_area,
            "solution area {:?} must be >= target area {:?}",
            solution.area,
            target_area
        );

        // Invariant 2: Tightness within directed rounding tolerance (at most 4 quanta)
        let diff = if solution.endpoint >= lo {
            solution.endpoint.checked_sub(lo).unwrap()
        } else {
            lo.checked_sub(solution.endpoint).unwrap()
        };
        assert!(
            diff.get().to_scaled_i128() <= 4,
            "endpoint {:?} must be within 4 quanta of expected {:?}",
            solution.endpoint,
            lo
        );

        // Invariant 3: Discrete Lattice Bound: 1 quantum beyond endpoint (narrower interval) must fall short of target_area
        let one_quantum = NonNeg::new(Fx::QUANTUM).unwrap();
        if let Ok(next_endpoint) = solution.endpoint.checked_add(one_quantum)
            && next_endpoint < hi
        {
            let narrower_width = hi.checked_sub(next_endpoint).unwrap();
            let narrower_area = calc
                .integrate(next_endpoint, narrower_width)
                .unwrap()
                .lower();
            assert!(
                narrower_area < target_area,
                "endpoint + QUANTUM must strictly fall below target area"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Interval & Clamping Constraints Fuzz Tests
// ---------------------------------------------------------------------------

#[test]
fn fuzz_solve_max_hi_with_hi_limit_constraints() {
    let mut fuzz = Fuzzer::new(0xbeef_cafe_1234_5678_8765_4321_abcd_ef01);
    let calc = LinearMonotone::new(nn(2), nn(5));

    for _ in 0..100 {
        let lo_raw = fuzz.nonnegative_raw(10);
        let lo = nn_raw(lo_raw);

        // hi_limit strictly less than lo must error
        if lo > NonNeg::zero() {
            let invalid_limit = lo.checked_sub(NonNeg::new(Fx::QUANTUM).unwrap()).unwrap();
            assert_eq!(
                calc.max_hi_for_area_at_or_below(lo, Some(invalid_limit), nn(10)),
                Err(MonotoneCalculatorError::InvalidInterval)
            );
        }

        // hi_limit == lo with non-zero area must return None
        assert_eq!(
            calc.max_hi_for_area_at_or_below(lo, Some(lo), nn(10))
                .unwrap(),
            None
        );

        // hi_limit between lo and the unconstrained solve must clamp the solution
        let width = nn(10);
        let area = calc.integrate(lo, width).unwrap().upper();
        let clamped_limit = lo.checked_add(nn(5)).unwrap();

        let clamped_solution = calc
            .max_hi_for_area_at_or_below(lo, Some(clamped_limit), area)
            .unwrap()
            .unwrap();

        assert_eq!(clamped_solution.endpoint, clamped_limit);
        assert!(clamped_solution.area <= area);
    }
}

// ---------------------------------------------------------------------------
// 4. Monotonicity Across Area Progression
// ---------------------------------------------------------------------------

#[test]
fn fuzz_monotone_monotonicity_across_areas() {
    let calc = LinearMonotone::new(nn(3), nn(4));
    let lo = nn(5);
    let hi = nn(20);

    let mut prev_up_endpoint = lo;
    let mut prev_up_area = NonNeg::zero();

    // Increasing areas must monotonically increase upward endpoint
    for a in 1..=40 {
        let area = nn(a * 10);
        let sol = calc
            .max_hi_for_area_at_or_below(lo, None, area)
            .unwrap()
            .unwrap();
        assert!(sol.endpoint >= prev_up_endpoint);
        assert!(sol.area >= prev_up_area);
        prev_up_endpoint = sol.endpoint;
        prev_up_area = sol.area;
    }

    let mut prev_down_endpoint = hi;
    let mut prev_down_area = NonNeg::zero();

    // Increasing targets must monotonically decrease downward endpoint (move leftwards)
    for a in 1..=40 {
        let area = nn(a * 10);
        if let Some(sol) = calc.max_lo_for_area_at_or_above(lo, hi, area).unwrap() {
            assert!(sol.endpoint <= prev_down_endpoint);
            assert!(sol.area >= prev_down_area);
            prev_down_endpoint = sol.endpoint;
            prev_down_area = sol.area;
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Boundary Conditions & Extreme Invariants
// ---------------------------------------------------------------------------

#[test]
fn boundary_conditions_on_dashu_grid() {
    let calc = LinearMonotone::new(nn(2), nn(3));
    let lo = nn(5);
    let hi = nn(10);

    // Zero area upward returns lo
    let zero_up = calc
        .max_hi_for_area_at_or_below(lo, None, NonNeg::zero())
        .unwrap()
        .unwrap();
    assert_eq!(zero_up.endpoint, lo);
    assert_eq!(zero_up.area, NonNeg::zero());

    // Zero area downward returns hi
    let zero_down = calc
        .max_lo_for_area_at_or_above(lo, hi, NonNeg::zero())
        .unwrap()
        .unwrap();
    assert_eq!(zero_down.endpoint, hi);
    assert_eq!(zero_down.area, NonNeg::zero());

    // Unreachable area downward (exceeding total area over [lo, hi]) returns None
    let total_area = calc
        .integrate(lo, hi.checked_sub(lo).unwrap())
        .unwrap()
        .lower();
    let unreachable = total_area.checked_add(nn(100)).unwrap();
    assert_eq!(
        calc.max_lo_for_area_at_or_above(lo, hi, unreachable)
            .unwrap(),
        None
    );

    // Downward solve with lo_limit > hi returns InvalidInterval
    assert_eq!(
        calc.max_lo_for_area_at_or_above(hi, lo, nn(10)),
        Err(MonotoneCalculatorError::InvalidInterval)
    );
}

// ---------------------------------------------------------------------------
// 6. Signed Enclosure (`require_nonnegative`) Fuzz Testing
// ---------------------------------------------------------------------------

#[test]
fn fuzz_require_nonnegative_invariants() {
    let mut fuzz = Fuzzer::new(0x3141_5926_5358_9793_2384_6264_3383_2795);

    for _ in 0..500 {
        let raw1 = (fuzz.next_u128() % (100 * RAW_ONE as u128)) as i128 - (50 * RAW_ONE);
        let raw2 = (fuzz.next_u128() % (100 * RAW_ONE as u128)) as i128 - (50 * RAW_ONE);

        let (lo_raw, hi_raw) = if raw1 <= raw2 {
            (raw1, raw2)
        } else {
            (raw2, raw1)
        };

        // Valid interval lo <= hi
        let bounds = Bounds::new(fx_raw(lo_raw), fx_raw(hi_raw)).unwrap();

        let result = require_nonnegative(bounds);
        if lo_raw < 0 {
            assert_eq!(result, Err(MonotoneCalculatorError::NegativeOutput));
        } else {
            let refined = result.unwrap();
            assert_eq!(refined.lower().get().to_scaled_i128(), lo_raw);
            assert_eq!(refined.upper().get().to_scaled_i128(), hi_raw);
        }

        // The numeric boundary now rejects inverted endpoints before a
        // calculator or solver receives the enclosure.
        if lo_raw < hi_raw {
            assert!(Bounds::new(fx_raw(hi_raw), fx_raw(lo_raw)).is_err());
        }
    }
}

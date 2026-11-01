//! High-precision differential oracle tests for `mayf-reward-clearing`.
//!
//! Evaluates `entry_weight` and `withdrawal_penalty` against:
//! 1. An independent arbitrary-precision rational oracle (`dashu::integer::IBig`).
//! 2. Cross-representation differential verification between binary fixed point
//!    (`I64F64`, Q64.64) and decimal fixed point (`DashuFixed12`, 1e-12).
//! 3. Strict defensive bounding:
//!    - `entry_weight` is mathematically bounded below the exact real value.
//!    - `withdrawal_penalty` is mathematically bounded above the exact real value.
//!    - User return is mathematically bounded below the continuous expectation.

use dashu::integer::IBig;
use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::FixedNumber;
use mayf_reward_clearing::*;

type Fx64 = I64F64;
type Fx12 = DashuFixed12;

const RAW_ONE_FX64: i128 = 1_i128 << Fx64::FRAC_BITS;
const RAW_ONE_FX12: i128 = 1_000_000_000_000;

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

    fn next_u64(&mut self) -> u64 {
        (self.next_u128() >> 64) as u64
    }

    fn range_u64(&mut self, min: u64, max: u64) -> u64 {
        assert!(min <= max);
        if min == max {
            return min;
        }
        let span = max - min + 1;
        min + (self.next_u64() % span)
    }

    fn biased_u64(&mut self, min: u64, max: u64) -> u64 {
        assert!(min <= max);
        if min == max {
            return min;
        }
        match self.next_u64() % 10 {
            0 => min,
            1 => min.saturating_add(1).min(max),
            2 => max.saturating_sub(1).max(min),
            3 => max,
            4 => min + (max - min) / 2,
            _ => self.range_u64(min, max),
        }
    }
}

/// Compute exact continuous entry weight in Q64.64:
/// W_exact = 1 + b * (T - tau) / T
/// Exact scaled value = (1 << 64) + floor( (b_raw * (T - tau)) / T )
fn exact_entry_weight_fx64(tau: u64, duration: u64, slope_raw_fx64: i128) -> i128 {
    let one = IBig::from(1) << 64;
    let bonus_exact =
        (IBig::from(slope_raw_fx64) * IBig::from(duration - tau)) / IBig::from(duration);
    (one + bonus_exact).try_into().unwrap()
}

/// Compute exact continuous withdrawal penalty in Q64.64 rounded up:
/// P_exact = (t / T)^k
/// P_exact_ceil = ceil( (t^k * 2^64) / T^k )
fn exact_withdrawal_penalty_fx64(t: u64, duration: u64, exponent: u32) -> i128 {
    if t == 0 {
        return 0;
    }
    if t == duration {
        return RAW_ONE_FX64;
    }
    let k = exponent as usize;
    let t_pow = IBig::from(t).pow(k);
    let dur_pow = IBig::from(duration).pow(k);
    let numerator = (t_pow * (IBig::from(1) << 64)) + &dur_pow - IBig::from(1);
    (numerator / dur_pow).try_into().unwrap()
}

#[test]
fn entry_weight_defensive_under_estimation_oracle() {
    let mut rng = Fuzzer::new(0x1122_3344_5566_7788);

    for _ in 0..10_000 {
        let duration_secs = rng.biased_u64(1, 1_000_000_000);
        let duration = EpochDuration::new(duration_secs).unwrap();

        // slope in [0, 2.0]
        let max_slope_raw = 2 * RAW_ONE_FX64;
        let slope_raw = (rng.next_u128() % (max_slope_raw as u128 + 1)) as i128;
        let slope = RampBonusSlope::<Fx64>::new(Fx64::from_scaled_i128(slope_raw)).unwrap();

        let tau = rng.biased_u64(0, duration_secs);

        let computed = entry_weight::<Fx64>(tau, duration, slope).unwrap();
        let computed_raw = computed.get().to_scaled_i128();

        let exact_raw = exact_entry_weight_fx64(tau, duration_secs, slope_raw);

        // Defensive invariant 1: computed weight must NEVER exceed the exact continuous weight
        assert!(
            computed_raw <= exact_raw,
            "Entry weight exceeded exact oracle: computed={computed_raw}, exact={exact_raw}, diff={}",
            computed_raw - exact_raw
        );

        // Tight precision error bound: drift must be at most 3 raw Q64 units (<= 1.6e-19)
        let drift = exact_raw - computed_raw;
        assert!(
            drift <= 3,
            "Entry weight drift from exact oracle exceeded 3 raw units: {drift}"
        );
    }
}

#[test]
fn withdrawal_penalty_defensive_over_estimation_oracle() {
    let mut rng = Fuzzer::new(0x8877_6655_4433_2211);

    for _ in 0..10_000 {
        let duration_secs = rng.biased_u64(1, 1_000_000_000);
        let duration = EpochDuration::new(duration_secs).unwrap();
        let exp_val = rng.range_u64(1, 4) as u32;
        let exponent = ExitPenaltyExponent::new(exp_val).unwrap();

        let t = rng.biased_u64(0, duration_secs);

        let computed = withdrawal_penalty::<Fx64>(t, duration, exponent).unwrap();
        let computed_raw = computed.get().to_scaled_i128();

        let exact_ceil_raw = exact_withdrawal_penalty_fx64(t, duration_secs, exp_val);

        // Defensive invariant 2: computed penalty must NEVER be less than the exact ceiling penalty
        assert!(
            computed_raw >= exact_ceil_raw,
            "Withdrawal penalty under-penalized: computed={computed_raw}, exact_ceil={exact_ceil_raw}"
        );

        // Tight precision error bound: compound ceiling multiplication drift <= 8 raw Q64 units
        let excess = computed_raw - exact_ceil_raw;
        assert!(
            excess <= 8,
            "Withdrawal penalty excess above exact ceiling exceeded 8 raw units: {excess}"
        );
    }
}

#[test]
fn user_return_strictly_defensive_against_continuous_oracle() {
    let mut rng = Fuzzer::new(0x3344_5566_7788_99aa);

    for _ in 0..5_000 {
        let duration_secs = rng.biased_u64(10, 100_000);
        let exp_val = rng.range_u64(1, 4) as u32;

        let window = EpochSchedule::try_new(0, duration_secs)
            .unwrap()
            .window_at(0)
            .unwrap();
        let terms =
            EpochTerms::try_new(duration_secs, Fx64::from_i32(1).unwrap(), exp_val).unwrap();
        let current_epoch = start_epoch(StartEpochArgs { window, terms }).unwrap().epoch;

        let user_pos = Position::new();

        let deposit_principal = rng.biased_u64(10, 10_000_000);
        let deposit_time = rng.biased_u64(0, duration_secs - 2);

        let dep_receipt = deposit(
            DepositState {
                epoch: current_epoch,
                position: user_pos,
                latest_lot: None,
            },
            DepositArgs {
                principal: deposit_principal,
                now: deposit_time,
            },
        )
        .unwrap();

        let withdraw_amount = rng.biased_u64(1, deposit_principal);
        let withdraw_time = rng.biased_u64(deposit_time, duration_secs - 1);

        let with_receipt = withdraw(
            WithdrawState {
                epoch: dep_receipt.epoch,
                position: dep_receipt.position,
                lot: dep_receipt.lot,
            },
            WithdrawArgs {
                principal: withdraw_amount,
                now: withdraw_time,
            },
        )
        .unwrap();

        let user_units = with_receipt.principal_returned;

        // Exact rational continuous user return: floor( x * ( 1 - (t/T)^k ) )
        let k = exp_val as usize;
        let t_pow = IBig::from(withdraw_time).pow(k);
        let dur_pow = IBig::from(duration_secs).pow(k);
        let exact_return_tokens: u64 = ((IBig::from(withdraw_amount) * (&dur_pow - &t_pow))
            / dur_pow)
            .try_into()
            .unwrap_or(0u64);

        // Defensive invariant 3: user units received <= exact continuous expectation
        assert!(
            user_units <= exact_return_tokens,
            "User received more than exact continuous expectation: got {user_units}, oracle {exact_return_tokens}"
        );
    }
}

#[test]
fn cross_representation_differential_fx64_vs_dashufx12() {
    let mut rng = Fuzzer::new(0x99aa_bbcc_ddee_ff00);

    for _ in 0..1_000 {
        let duration_secs = rng.biased_u64(10, 1_000_000);
        let duration = EpochDuration::new(duration_secs).unwrap();
        let exp_val = rng.range_u64(1, 4) as u32;
        let exponent = ExitPenaltyExponent::new(exp_val).unwrap();

        // Test slope = 1.0 (both representations have exact 1.0)
        let slope_fx64 = RampBonusSlope::<Fx64>::new(Fx64::from_i32(1).unwrap()).unwrap();
        let slope_fx12 = RampBonusSlope::<Fx12>::new(Fx12::from_scaled_i128(RAW_ONE_FX12)).unwrap();

        let tau = rng.biased_u64(0, duration_secs);
        let t = rng.biased_u64(0, duration_secs);

        let w_fx64 = entry_weight::<Fx64>(tau, duration, slope_fx64).unwrap();
        let w_fx12 = entry_weight::<Fx12>(tau, duration, slope_fx12).unwrap();

        let p_fx64 = withdrawal_penalty::<Fx64>(t, duration, exponent).unwrap();
        let p_fx12 = withdrawal_penalty::<Fx12>(t, duration, exponent).unwrap();

        // Convert both to continuous normalized float f64 to test differential agreement
        let w_f64_from_fx64 = (w_fx64.get().to_scaled_i128() as f64) / (RAW_ONE_FX64 as f64);
        let w_f64_from_fx12 = (w_fx12.get().to_scaled_i128() as f64) / (RAW_ONE_FX12 as f64);

        let p_f64_from_fx64 = (p_fx64.get().to_scaled_i128() as f64) / (RAW_ONE_FX64 as f64);
        let p_f64_from_fx12 = (p_fx12.get().to_scaled_i128() as f64) / (RAW_ONE_FX12 as f64);

        // Both representations must agree to within DashuFixed12 resolution (< 1e-11)
        let w_diff = (w_f64_from_fx64 - w_f64_from_fx12).abs();
        assert!(
            w_diff < 1e-11,
            "Cross-representation weight difference exceeded 1e-11: {w_diff} (fx64={w_f64_from_fx64}, fx12={w_f64_from_fx12})"
        );

        let p_diff = (p_f64_from_fx64 - p_f64_from_fx12).abs();
        assert!(
            p_diff < 1e-11,
            "Cross-representation penalty difference exceeded 1e-11: {p_diff} (fx64={p_f64_from_fx64}, fx12={p_f64_from_fx12})"
        );
    }
}

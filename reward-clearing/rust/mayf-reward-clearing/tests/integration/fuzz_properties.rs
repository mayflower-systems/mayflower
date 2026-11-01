//! Deterministic property and fuzz tests for `mayf-reward-clearing`.
//!
//! Evaluates protocol-level mathematical invariants:
//! 1. Monotonicity and bounds of entry weights ($w_b(\tau) \in [1, 1+b]$ non-increasing).
//! 2. Monotonicity, convexity, and bounds of exit penalties ($p_k(t) \in [0, 1]$ non-decreasing).
//! 3. Strict defensive principal conservation during withdrawal ($x_{user} + x_{burn} == x$).
//! 4. Non-increasing share density of remaining capital during partial lot withdrawals.
//! 5. Pro-rata reward settlement solvency ($\sum payout \le R$), dust boundedness, and claim order invariance.
//! 6. Numeric lot and aggregate consistency over caller-selected sequences.
//! 7. Extreme edge cases: minimum/maximum durations, boundary times, extreme capital amounts.

use dashu::integer::IBig;
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::NonNeg;
use mayf_reward_clearing::*;

type Number = I64F64;
type Amount = NonNeg<Number>;

const RAW_ONE: i128 = 1_i128 << Number::FRAC_BITS;

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

    fn slope_raw(&mut self) -> i128 {
        // RampBonusSlope is in [0, 2.0]
        let max_raw = 2 * RAW_ONE;
        match self.next_u64() % 6 {
            0 => 0,
            1 => RAW_ONE,
            2 => max_raw,
            3 => RAW_ONE / 2,
            4 => (3 * RAW_ONE) / 2,
            _ => (self.next_u128() % (max_raw as u128 + 1)) as i128,
        }
    }

    fn exponent(&mut self) -> u32 {
        (self.range_u64(1, 4)) as u32
    }
}

fn number_from_scaled(raw: i128) -> Number {
    Number::from_scaled_i128(raw)
}

fn amount_from_scaled(raw: i128) -> Amount {
    NonNeg::new(number_from_scaled(raw)).unwrap()
}

fn make_epoch(
    duration_secs: u64,
    slope: RampBonusSlope<Number>,
    exponent: ExitPenaltyExponent,
) -> Epoch<Number> {
    let window = EpochSchedule::try_new(0, duration_secs)
        .unwrap()
        .window_at(0)
        .unwrap();
    let terms = EpochTerms::try_new(duration_secs, slope.get().get(), exponent.get()).unwrap();
    start_epoch(StartEpochArgs { window, terms }).unwrap().epoch
}

#[test]
fn entry_weight_monotonicity_and_defensive_bounds() {
    let mut rng = Fuzzer::new(0xdead_beef_1234_5678);

    for _ in 0..10_000 {
        let duration_secs = rng.biased_u64(1, 1_000_000_000);
        let duration = EpochDuration::new(duration_secs).unwrap();
        let slope_raw = rng.slope_raw();
        let slope = RampBonusSlope::new(number_from_scaled(slope_raw)).unwrap();

        let tau1 = rng.biased_u64(0, duration_secs);
        let tau2 = rng.biased_u64(0, duration_secs);
        let (t_early, t_late) = if tau1 <= tau2 {
            (tau1, tau2)
        } else {
            (tau2, tau1)
        };

        let w_early = entry_weight::<Number>(t_early, duration, slope).unwrap();
        let w_late = entry_weight::<Number>(t_late, duration, slope).unwrap();

        // Monotonicity: earlier deposit gets greater or equal weight
        assert!(
            w_early >= w_late,
            "Entry weight monotonicity violated: w({t_early}) = {:?} < w({t_late}) = {:?}",
            w_early.get().to_scaled_i128(),
            w_late.get().to_scaled_i128()
        );

        // Lower bound: w(tau) >= 1.0
        let one = amount_from_scaled(RAW_ONE);
        assert!(
            w_late >= one,
            "Entry weight fell below 1.0: {:?}",
            w_late.get().to_scaled_i128()
        );

        // Upper bound: w(tau) <= 1.0 + b
        let max_possible = amount_from_scaled(RAW_ONE + slope_raw);
        assert!(
            w_early <= max_possible,
            "Entry weight exceeded 1.0 + b: {:?} > {:?}",
            w_early.get().to_scaled_i128(),
            max_possible.get().to_scaled_i128()
        );

        // Terminal condition: at tau == duration, weight is exactly 1.0
        let w_end = entry_weight::<Number>(duration_secs, duration, slope).unwrap();
        assert_eq!(
            w_end, one,
            "Terminal weight at duration must be exactly 1.0"
        );
    }
}

#[test]
fn withdrawal_penalty_monotonicity_and_bounds() {
    let mut rng = Fuzzer::new(0xbeef_cafe_8765_4321);

    for _ in 0..10_000 {
        let duration_secs = rng.biased_u64(1, 1_000_000_000);
        let duration = EpochDuration::new(duration_secs).unwrap();
        let exp_val = rng.exponent();
        let exponent = ExitPenaltyExponent::new(exp_val).unwrap();

        let t1 = rng.biased_u64(0, duration_secs);
        let t2 = rng.biased_u64(0, duration_secs);
        let (t_early, t_late) = if t1 <= t2 { (t1, t2) } else { (t2, t1) };

        let p_early = withdrawal_penalty::<Number>(t_early, duration, exponent).unwrap();
        let p_late = withdrawal_penalty::<Number>(t_late, duration, exponent).unwrap();

        // Monotonicity: later withdrawal suffers greater or equal penalty
        assert!(
            p_early <= p_late,
            "Penalty monotonicity violated: p({t_early}) = {:?} > p({t_late}) = {:?}",
            p_early.get().to_scaled_i128(),
            p_late.get().to_scaled_i128()
        );

        // Bounds: 0 <= p(t) <= 1
        let zero = amount_from_scaled(0);
        let one = amount_from_scaled(RAW_ONE);
        assert!(
            p_early >= zero && p_late <= one,
            "Penalty out of bounds [0, 1]"
        );

        // Boundary values
        let p_zero = withdrawal_penalty::<Number>(0, duration, exponent).unwrap();
        assert_eq!(p_zero, zero, "Penalty at t=0 must be exactly 0");

        let p_end = withdrawal_penalty::<Number>(duration_secs, duration, exponent).unwrap();
        assert_eq!(p_end, one, "Penalty at t=duration must be exactly 1");
    }
}

#[test]
fn penalty_convexity_for_all_exponents() {
    let duration_secs = 10_000;
    let duration = EpochDuration::new(duration_secs).unwrap();

    for k in 1..=4 {
        let exponent = ExitPenaltyExponent::new(k).unwrap();
        let step = 500;
        for t in (step..(duration_secs - step)).step_by(step as usize) {
            let p_prev = withdrawal_penalty::<Number>(t - step, duration, exponent)
                .unwrap()
                .get()
                .to_scaled_i128();
            let p_curr = withdrawal_penalty::<Number>(t, duration, exponent)
                .unwrap()
                .get()
                .to_scaled_i128();
            let p_next = withdrawal_penalty::<Number>(t + step, duration, exponent)
                .unwrap()
                .get()
                .to_scaled_i128();

            let diff1 = p_curr - p_prev;
            let diff2 = p_next - p_curr;

            // diff2 >= diff1 - 10 epsilon for discretization
            assert!(
                diff2 >= diff1 - 10,
                "Convexity failed for k={k} at t={t}: diff1={diff1}, diff2={diff2}"
            );
        }
    }
}

#[test]
fn withdrawal_exact_principal_conservation() {
    let mut rng = Fuzzer::new(0x4321_8765_feed_face);

    for _ in 0..5_000 {
        let duration_secs = rng.biased_u64(10, 100_000);
        let slope_raw = rng.slope_raw();
        let slope = RampBonusSlope::new(number_from_scaled(slope_raw)).unwrap();
        let exponent = ExitPenaltyExponent::new(rng.exponent()).unwrap();

        let mut current_epoch = make_epoch(duration_secs, slope, exponent);

        let user_pos = Position::new();

        let deposit_principal = rng.biased_u64(1, 100_000_000);
        let deposit_time = rng.biased_u64(0, duration_secs - 1);

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

        current_epoch = dep_receipt.epoch;
        let mut position = dep_receipt.position;
        let lot = dep_receipt.lot;

        // Choose withdrawal amount and time
        let withdraw_amount = rng.biased_u64(1, deposit_principal);
        let withdraw_time = rng.biased_u64(deposit_time, duration_secs - 1);

        let with_receipt = withdraw(
            WithdrawState {
                epoch: current_epoch,
                position,
                lot,
            },
            WithdrawArgs {
                principal: withdraw_amount,
                now: withdraw_time,
            },
        )
        .unwrap();

        // Conservation invariant: returned + burned == principal withdrawn.
        // Both halves are whole token units, so this equality also proves the
        // split left no fractional residue behind.
        let to_user = i128::from(with_receipt.principal_returned);
        let to_burn = i128::from(with_receipt.principal_forfeited);
        let withdrawn = i128::from(with_receipt.principal_withdrawn);

        assert_eq!(
            to_user + to_burn,
            withdrawn,
            "Withdrawal did not conserve principal: user={to_user}, burn={to_burn}, total={withdrawn}"
        );

        // User returned tokens must be <= floor(x * (1 - (t/T)^k))
        let user_units = with_receipt.principal_returned;
        let elapsed = withdraw_time;
        let k = exponent.get() as usize;
        let exact_penalty_num = IBig::from(elapsed).pow(k);
        let exact_penalty_den = IBig::from(duration_secs).pow(k);

        let max_continuous_user_return: u64 = ((IBig::from(withdraw_amount)
            * (exact_penalty_den.clone() - exact_penalty_num))
            / exact_penalty_den)
            .try_into()
            .unwrap_or(0u64);

        assert!(
            user_units <= max_continuous_user_return + 1, // allow at most 1 unit rounding slack
            "User received more than continuous expectation: got {user_units}, max {max_continuous_user_return}"
        );

        // Position principal is updated accurately
        position = with_receipt.position;
        let expected_pos_principal = deposit_principal - withdraw_amount;
        assert_eq!(
            position.principal(),
            expected_pos_principal,
            "User position principal mismatch after withdrawal"
        );
    }
}

#[test]
fn withdrawal_share_density_is_strictly_non_increasing() {
    let mut rng = Fuzzer::new(0x9876_5432_10fe_dcba);

    for _ in 0..5_000 {
        let duration_secs = rng.biased_u64(10, 100_000);
        let slope_raw = rng.slope_raw();
        let slope = RampBonusSlope::new(number_from_scaled(slope_raw)).unwrap();
        let exponent = ExitPenaltyExponent::new(rng.exponent()).unwrap();

        let current_epoch = make_epoch(duration_secs, slope, exponent);

        let user_pos = Position::new();

        let deposit_principal = rng.biased_u64(2, 100_000_000);
        let deposit_time = rng.biased_u64(0, duration_secs - 1);

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

        let initial_lot = dep_receipt.lot;
        let withdraw_amount = rng.biased_u64(1, deposit_principal - 1); // strictly partial withdrawal

        let with_receipt = withdraw(
            WithdrawState {
                epoch: dep_receipt.epoch,
                position: dep_receipt.position,
                lot: initial_lot,
            },
            WithdrawArgs {
                principal: withdraw_amount,
                now: deposit_time,
            },
        )
        .unwrap();

        let remaining_lot = with_receipt.lot.expect("partial withdrawal leaves lot");

        // Verify share density: (S' / A') <= (S / A)  <=>  S' * A <= S * A'
        let s_initial = initial_lot.shares().get().to_scaled_i128();
        let a_initial = initial_lot.principal();
        let s_remaining = remaining_lot.shares().get().to_scaled_i128();
        let a_remaining = remaining_lot.principal();

        let lhs = IBig::from(s_remaining) * IBig::from(a_initial);
        let rhs = IBig::from(s_initial) * IBig::from(a_remaining);

        assert!(
            lhs <= rhs,
            "Partial withdrawal inflated share density! lhs={lhs}, rhs={rhs}"
        );
    }
}

#[test]
fn pro_rata_settlement_solvency_dust_bound_and_order_invariance() {
    let mut rng = Fuzzer::new(0x1357_9bdf_2468_ace0);

    for _ in 0..1_000 {
        let duration_secs = rng.biased_u64(10, 10_000);
        let slope = RampBonusSlope::new(number_from_scaled(rng.slope_raw())).unwrap();
        let exponent = ExitPenaltyExponent::new(rng.exponent()).unwrap();

        let mut current_epoch = make_epoch(duration_secs, slope, exponent);
        let num_users = rng.range_u64(2, 12) as usize;

        let mut positions = Vec::new();

        for _ in 0..num_users {
            let pos = Position::new();
            let principal = rng.biased_u64(1, 10_000_000);
            let time = rng.biased_u64(0, duration_secs - 1);

            let dep = deposit(
                DepositState {
                    epoch: current_epoch,
                    position: pos,
                    latest_lot: None,
                },
                DepositArgs {
                    principal,
                    now: time,
                },
            )
            .unwrap();

            current_epoch = dep.epoch;
            positions.push(dep.position);
        }

        // Add reward to pot
        let reward_pot_amount = rng.biased_u64(1, 1_000_000_000);
        let add_rec = add_reward(
            AddRewardState {
                epoch: current_epoch,
            },
            AddRewardArgs {
                amount: reward_pot_amount,
                now: duration_secs - 1,
            },
        )
        .unwrap();
        current_epoch = add_rec.epoch;

        // Settle epoch
        let settle_rec = settle(
            SettleState {
                epoch: current_epoch,
            },
            SettleArgs { now: duration_secs },
        )
        .unwrap();
        let finalized_epoch = settle_rec.epoch;

        // Claim in standard sequential order
        let mut final_a = finalized_epoch;
        let mut payouts_a = Vec::new();
        for &position in &positions {
            let claim_rec = claim(
                ClaimState {
                    epoch: final_a,
                    position,
                },
                ClaimArgs { now: duration_secs },
            )
            .unwrap();
            payouts_a.push(i128::from(claim_rec.reward_earned));
            final_a = claim_rec.epoch;
        }

        // Claim in reverse order
        let mut final_b = finalized_epoch;
        let mut payouts_b = vec![0i128; num_users];
        for i in (0..num_users).rev() {
            let claim_rec = claim(
                ClaimState {
                    epoch: final_b,
                    position: positions[i],
                },
                ClaimArgs { now: duration_secs },
            )
            .unwrap();
            payouts_b[i] = i128::from(claim_rec.reward_earned);
            final_b = claim_rec.epoch;
        }

        // Invariant 1: Order Invariance (Commutativity)
        for i in 0..num_users {
            assert_eq!(
                payouts_a[i], payouts_b[i],
                "Claim payout differed by execution ordering for user {i}!"
            );
        }

        // Invariant 2: Total Payout Solvency (no insolvency)
        let total_payout: i128 = payouts_a.iter().sum();
        assert!(
            total_payout <= reward_pot_amount as i128,
            "Total claims exceeded available reward pot! {total_payout} > {reward_pot_amount}"
        );

        // Invariant 3: Dust Bound
        let dust = (reward_pot_amount as i128) - total_payout;
        assert!(
            dust < num_users as i128,
            "Dust left in pot exceeded number of claimants: dust={dust}, users={num_users}"
        );

        // Invariant 4: Remainder is available after all claims
        assert_eq!(
            final_a.unclaimed_shares(),
            amount_from_scaled(0),
            "Unclaimed shares must be 0 after all users claim"
        );
        let remaining_reward = i128::from(final_a.remaining_reward().unwrap());
        assert_eq!(
            remaining_reward, dust,
            "Final reward did not match the rounding remainder"
        );
    }
}

#[test]
fn selected_lot_sequence_preserves_principal_and_share_aggregates() {
    let duration_secs = 10_000;
    let slope = RampBonusSlope::new(number_from_scaled(RAW_ONE)).unwrap(); // slope = 1.0
    let exponent = ExitPenaltyExponent::new(2).unwrap();

    let mut current_epoch = make_epoch(duration_secs, slope, exponent);

    let mut user_pos = Position::new();

    let num_lots = 5;
    let mut lots: Vec<Lot<Number>> = Vec::new();
    let mut current_time = 100;

    // Push 5 lots at strictly increasing times
    for _ in 0..num_lots {
        let principal = 1000;
        let dep = deposit(
            DepositState {
                epoch: current_epoch,
                position: user_pos,
                latest_lot: lots.last().copied(),
            },
            DepositArgs {
                principal,
                now: current_time,
            },
        )
        .unwrap();

        // Invariant: newer lot weight must be <= previous lot weight
        if let Some(prev) = lots.last() {
            assert!(
                dep.lot.entry_weight() <= prev.entry_weight(),
                "LIFO lot stack violated monotonic non-increasing entry weight"
            );
        }

        current_epoch = dep.epoch;
        user_pos = dep.position;
        lots.push(dep.lot);
        current_time += 500;
    }

    // Now withdraw LIFO
    while let Some(top_lot) = lots.pop() {
        let remaining_principal = top_lot.principal();
        let with = withdraw(
            WithdrawState {
                epoch: current_epoch,
                position: user_pos,
                lot: top_lot,
            },
            WithdrawArgs {
                principal: remaining_principal,
                now: current_time,
            },
        )
        .unwrap();

        assert!(
            with.lot.is_none(),
            "Full lot withdrawal must consume the lot completely"
        );
        current_epoch = with.epoch;
        user_pos = with.position;
    }

    // After popping all lots, position principal and shares must be zero
    assert_eq!(user_pos.principal(), 0);
    assert_eq!(user_pos.shares(), amount_from_scaled(0));
}

#[test]
fn extreme_numerical_and_temporal_boundaries() {
    // 1. Duration = 1 second
    let slope = RampBonusSlope::new(number_from_scaled(2 * RAW_ONE)).unwrap(); // slope = 2.0
    let exp = ExitPenaltyExponent::new(4).unwrap();
    let d1 = EpochDuration::new(1).unwrap();
    let w0 = entry_weight::<Number>(0, d1, slope).unwrap();
    let w1 = entry_weight::<Number>(1, d1, slope).unwrap();
    assert_eq!(w0, amount_from_scaled(3 * RAW_ONE));
    assert_eq!(w1, amount_from_scaled(RAW_ONE));

    let p0 = withdrawal_penalty::<Number>(0, d1, exp).unwrap();
    let p1 = withdrawal_penalty::<Number>(1, d1, exp).unwrap();
    assert_eq!(p0, amount_from_scaled(0));
    assert_eq!(p1, amount_from_scaled(RAW_ONE));

    // 2. Duration = u32::MAX
    let d_max = EpochDuration::new(u32::MAX as u64).unwrap();
    let w_mid = entry_weight::<Number>((u32::MAX as u64) / 2, d_max, slope).unwrap();
    assert!(w_mid > amount_from_scaled(RAW_ONE) && w_mid < amount_from_scaled(3 * RAW_ONE));

    // 3. Slope = 0 (Flat ramp)
    let s0 = RampBonusSlope::new(number_from_scaled(0)).unwrap();
    let wf0 = entry_weight::<Number>(0, d_max, s0).unwrap();
    let wf_end = entry_weight::<Number>(u32::MAX as u64, d_max, s0).unwrap();
    assert_eq!(wf0, amount_from_scaled(RAW_ONE));
    assert_eq!(wf_end, amount_from_scaled(RAW_ONE));

    // 4. Large capital amounts: 10^12 token units
    let large_principal = 1_000_000_000_000u64; // 1 trillion units
    let ep = make_epoch(1000, slope, exp);

    let pos = Position::new();
    let dep = deposit(
        DepositState {
            epoch: ep,
            position: pos,
            latest_lot: None,
        },
        DepositArgs {
            principal: large_principal,
            now: 0,
        },
    )
    .unwrap();
    // At tau=0 with slope=2, shares issued = 3 * principal
    let expected_shares = (3 * large_principal) as i128 * RAW_ONE;
    assert_eq!(
        dep.shares_issued.get().to_scaled_i128(),
        expected_shares,
        "Large deposit shares calculation mismatch"
    );
}

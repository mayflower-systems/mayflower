//! A cheap, continuously declining approximation, evaluated at whole seconds.

use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::Result;

/// The schedule uses a fixed 365-day year, independent of calendar dates.
pub const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;

/// Current emission rate and its annual decay coefficient.
///
/// Rates are token base units per second. A coefficient `d` defines the
/// per-second retention factor `q = Y / (Y + d)`, where `Y` is
/// [`SECONDS_PER_YEAR`]. This inexpensive recurrence approximates continuous
/// exponential decay; it does not evaluate an exponential or promise an exact
/// annual percentage decrease.
///
/// A second pays its *ending* rate. After `n` seconds, the new rate is
/// `rate * q^n` and the budget is `rate * (q + ... + q^n)`. All arithmetic
/// rounds down. There are no fractional carries, and repeated checkpoints can
/// discard additional fixed-point dust.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Schedule<N> {
    pub emission_rate: NonNeg<N>,
    pub annual_decay_rate: NonNeg<N>,
}

impl<N: FixedNumber> Schedule<N> {
    /// Advance the schedule and return its precise, conservatively rounded budget.
    ///
    /// The host supplies elapsed whole seconds. Work is bounded by 64 binary
    /// doubling steps, independently of the duration. The returned budget is
    /// still on `N`'s grid; gauge settlement performs the whole-token truncation.
    pub fn advance(self, elapsed_seconds: u64) -> Result<(Self, N)> {
        let rate = self.emission_rate.get();
        if elapsed_seconds == 0 || rate == N::ZERO {
            return Ok((self, N::ZERO));
        }
        let decay = self.annual_decay_rate.get();
        if decay == N::ZERO {
            let budget = rate.checked_mul_floor(N::checked_from_u64(elapsed_seconds)?)?;
            return Ok((self, budget));
        }

        let year = N::checked_from_u64(SECONDS_PER_YEAR)?;
        let retention = year.checked_div_floor(year.checked_add(decay)?)?;
        let (attenuation, duration) = geometric_interval(retention, elapsed_seconds)?;
        let budget = rate.checked_mul_floor(duration)?;
        let next_rate = rate.checked_mul_floor(attenuation)?;
        Ok((
            Self {
                emission_rate: NonNeg::new(next_rate)?,
                ..self
            },
            budget,
        ))
    }
}

/// Concatenate intervals `(a, s)` using `(a*b, s + a*t)`. With nonnegative
/// values, flooring each operation preserves a lower bound on both the ending
/// rate factor and the sum of the per-second rate factors.
fn geometric_interval<N: FixedNumber>(retention: N, mut seconds: u64) -> Result<(N, N)> {
    let mut attenuation = N::ONE;
    let mut duration = N::ZERO;
    let mut block_attenuation = retention;
    let mut block_duration = retention;

    while seconds != 0 {
        if seconds & 1 != 0 {
            duration = duration.checked_add(attenuation.checked_mul_floor(block_duration)?)?;
            attenuation = attenuation.checked_mul_floor(block_attenuation)?;
        }
        seconds >>= 1;
        if seconds != 0 {
            block_duration =
                block_duration.checked_add(block_attenuation.checked_mul_floor(block_duration)?)?;
            block_attenuation = block_attenuation.checked_mul_floor(block_attenuation)?;
        }
    }
    Ok((attenuation, duration))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mayf_fixed_drivers::U192F64;
    use mayf_fixed_number::{DirectedRounding, IntegralRounding};

    type Number = U192F64;

    fn schedule(rate: u64, decay: u64) -> Schedule<Number> {
        Schedule {
            emission_rate: NonNeg::from_unsigned(Number::from_u64(rate)),
            annual_decay_rate: NonNeg::from_unsigned(Number::from_u64(decay)),
        }
    }

    #[test]
    fn zero_duration_zero_rate_and_zero_decay_are_exact() {
        let initial = schedule(7, SECONDS_PER_YEAR);
        assert_eq!(initial.advance(0).unwrap(), (initial, Number::ZERO));

        let stopped = schedule(0, u64::MAX);
        assert_eq!(stopped.advance(u64::MAX).unwrap(), (stopped, Number::ZERO));

        let constant = schedule(7, 0);
        assert_eq!(
            constant.advance(19).unwrap(),
            (constant, Number::from_u64(133))
        );
    }

    #[test]
    fn seconds_pay_the_ending_rate() {
        // d = Y gives exact retention 1/2: the four payments are 8, 4, 2, 1.
        let (next, budget) = schedule(16, SECONDS_PER_YEAR).advance(4).unwrap();
        assert_eq!(next.emission_rate.get(), Number::ONE);
        assert_eq!(budget, Number::from_u64(15));
    }

    #[test]
    fn binary_intervals_match_an_independent_integer_geometric_sum() {
        // d = Y/3 gives retention 3/4. Scaling the initial rate by 4^n
        // makes every expected payment integral, so the oracle needs no fixed math.
        for seconds in 1u32..=20 {
            let rate = 4u64.pow(seconds);
            let expected = (1..=seconds)
                .map(|i| 3u64.pow(i) * 4u64.pow(seconds - i))
                .sum::<u64>();
            let (next, budget) = schedule(rate, SECONDS_PER_YEAR / 3)
                .advance(u64::from(seconds))
                .unwrap();
            assert_eq!(budget, Number::from_u64(expected));
            assert_eq!(
                next.emission_rate.get(),
                Number::from_u64(3u64.pow(seconds))
            );
        }
    }

    #[test]
    fn non_binary_retention_never_exceeds_the_exact_rational_budget() {
        // d = Y/2 gives ideal retention 2/3. For n seconds:
        // sum(q^i, i=1..n) = 2 * (3^n - 2^n) / 3^n.
        for seconds in 1u32..=20 {
            let denominator = 3u128.pow(seconds);
            let numerator = 2 * (denominator - 2u128.pow(seconds));
            let expected = Number::checked_from_ratio_round(
                numerator as i128,
                core::num::NonZeroU128::new(denominator).unwrap(),
                DirectedRounding::Floor,
            )
            .unwrap();
            let (_, actual) = schedule(1, SECONDS_PER_YEAR / 2)
                .advance(u64::from(seconds))
                .unwrap();
            assert!(actual <= expected);
            let allowance = Number::QUANTUM
                .checked_mul_floor(Number::from_u64(128))
                .unwrap();
            assert!(expected.checked_sub(actual).unwrap() <= allowance);
        }
    }

    #[test]
    fn ordinary_annual_decay_matches_an_independent_decimal_reference() {
        let initial = Schedule {
            emission_rate: NonNeg::from_unsigned(Number::from_u64(1_000_000)),
            annual_decay_rate: NonNeg::from_unsigned(
                Number::ONE.checked_div_floor(Number::from_u64(10)).unwrap(),
            ),
        };
        // Independent 100-digit decimal evaluation of r*q^n and
        // r*q*(1-q^n)/(1-q), with q quantized to the driver's 2^-64 grid.
        // One year: rate 904837.4181779..., budget 30010471802913.27788...
        // Ten years: rate 367879.4417486..., budget 199345539246842.64839...
        for (years, expected_rate, expected_budget) in [
            (1, 904_837, 30_010_471_802_913),
            (10, 367_879, 199_345_539_246_842),
        ] {
            let (next, budget) = initial.advance(years * SECONDS_PER_YEAR).unwrap();
            assert_eq!(
                next.emission_rate.get().checked_floor_to_u64().unwrap(),
                expected_rate
            );
            let actual = budget.checked_floor_to_u64().unwrap();
            assert!(actual <= expected_budget);
            // Powers can lose at most (n-1) quanta. Concatenating geometric
            // intervals then loses at most n*(n-1)/2 quanta in their sum.
            // Multiply by the integer rate and round the allowed loss upward.
            let n = u128::from(years * SECONDS_PER_YEAR);
            let allowance = (1_000_000 * n * (n - 1) / 2).div_ceil(1 << 64);
            assert!(u128::from(expected_budget - actual) <= allowance);
        }
    }

    #[test]
    fn the_full_u64_duration_needs_no_tick_loop() {
        let (next, budget) = schedule(16, SECONDS_PER_YEAR).advance(u64::MAX).unwrap();
        assert_eq!(next.emission_rate.get(), Number::ZERO);
        assert!(budget < Number::from_u64(16));
        assert_eq!(budget.checked_floor_to_u64().unwrap(), 15);
    }

    #[test]
    fn long_lived_constant_rate_uses_the_wide_accounting_range() {
        let (next, budget) = schedule(u64::MAX, 0).advance(u64::MAX).unwrap();
        assert_eq!(next, schedule(u64::MAX, 0));
        assert_eq!(
            budget,
            Number::from_u64(u64::MAX)
                .checked_mul_floor(Number::from_u64(u64::MAX))
                .unwrap()
        );
        assert!(budget.checked_floor_to_u64().is_err());
    }
}

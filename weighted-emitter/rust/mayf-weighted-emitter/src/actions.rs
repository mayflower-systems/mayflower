//! Actions consume model drafts and return all changed records plus whole-unit
//! allocation effects. The host commits the entire receipt, or none of it.

mod create_emitter;
mod create_gauge;
mod emit_to_gauge;
mod set_emitter_decay_rate;
mod set_gauge_votes;

use mayf_fixed_number::{FixedNumber, IntegralRounding, NonNeg};

use crate::{Amount, Emitter, Gauge, Result, Seconds, WeightedEmitterError};

pub use create_emitter::{CreateEmitterArgs, CreateEmitterReceipt, create_emitter};
pub use create_gauge::{CreateGaugeArgs, CreateGaugeReceipt, create_gauge};
pub use emit_to_gauge::{EmitToGaugeArgs, emit_to_gauge};
pub use set_emitter_decay_rate::{
    SetEmitterDecayRateArgs, SetEmitterDecayRateReceipt, set_emitter_decay_rate,
};
pub use set_gauge_votes::{SetGaugeVotesArgs, set_gauge_votes};

/// Shared result of settlement and reweighting. `emitted` is already included
/// in both cumulative totals. It is an allocation, not a token-mint instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GaugeReceipt<I, N: FixedNumber> {
    pub emitter_next: Emitter<I, N>,
    pub gauge_next: Gauge<I, N>,
    pub emitted: Amount,
}

/// Accrue the elapsed interval under the old weights. Empty intervals are
/// skipped permanently; fractional index dust is discarded without a carry.
fn advance<I, N: FixedNumber>(mut emitter: Emitter<I, N>, at: Seconds) -> Result<Emitter<I, N>> {
    let elapsed = at
        .checked_sub(emitter.last_updated_at_unix_seconds)
        .ok_or(WeightedEmitterError::TimeWentBack)?;
    let (schedule, budget) = emitter.schedule.advance(elapsed)?;
    emitter.schedule = schedule;
    if emitter.total_gauge_weight != 0 {
        let increment =
            budget.checked_div_floor(N::checked_from_u64(emitter.total_gauge_weight)?)?;
        emitter.emission_index = NonNeg::new(emitter.emission_index.get().checked_add(increment)?)?;
    }
    emitter.last_updated_at_unix_seconds = at;
    Ok(emitter)
}

/// Check related records before advancing: a future checkpoint must not become
/// valid merely because this call would advance the emitter beyond it.
fn settle<I: Copy + Eq, N: IntegralRounding>(
    emitter: Emitter<I, N>,
    mut gauge: Gauge<I, N>,
    at: Seconds,
) -> Result<GaugeReceipt<I, N>> {
    if gauge.emitter_id != emitter.id {
        return Err(WeightedEmitterError::GaugeEmitterMismatch);
    }
    if gauge.votes > emitter.total_gauge_weight
        || gauge.emission_index_seen > emitter.emission_index
        || gauge.total_emitted > emitter.total_emitted
        || gauge.last_updated_at_unix_seconds > emitter.last_updated_at_unix_seconds
    {
        return Err(WeightedEmitterError::InvalidState);
    }
    let mut emitter = advance(emitter, at)?;
    let growth = emitter
        .emission_index
        .get()
        .checked_sub(gauge.emission_index_seen.get())?;
    let emitted = growth
        .checked_mul_floor(N::checked_from_u64(gauge.votes)?)?
        .checked_floor_to_u64()?;
    gauge.total_emitted = gauge
        .total_emitted
        .checked_add(emitted)
        .ok_or(WeightedEmitterError::ArithmeticOverflow)?;
    emitter.total_emitted = emitter
        .total_emitted
        .checked_add(emitted)
        .ok_or(WeightedEmitterError::ArithmeticOverflow)?;
    // Advance the entire checkpoint even when the whole-unit payout is zero.
    gauge.emission_index_seen = emitter.emission_index;
    gauge.last_updated_at_unix_seconds = at;
    Ok(GaugeReceipt {
        emitter_next: emitter,
        gauge_next: gauge,
        emitted,
    })
}

#[cfg(test)]
mod tests {
    use mayf_fixed_drivers::U192F64;

    use super::*;
    use crate::Schedule;

    fn amount(value: u64) -> NonNeg<U192F64> {
        NonNeg::from_unsigned(U192F64::from_u64(value))
    }

    /// Explicit canonical inputs keep creation and prior transitions out of the
    /// action under test. The gauge has eight units awaiting materialization.
    fn records() -> (Emitter<u64, U192F64>, Gauge<u64, U192F64>) {
        (
            Emitter {
                id: 1,
                schedule: Schedule {
                    emission_rate: amount(120),
                    annual_decay_rate: amount(0),
                },
                last_updated_at_unix_seconds: 10,
                total_emitted: 100,
                total_gauge_weight: 6,
                emission_index: amount(10),
            },
            Gauge {
                id: 2,
                emitter_id: 1,
                votes: 4,
                emission_index_seen: amount(8),
                total_emitted: 20,
                last_updated_at_unix_seconds: 8,
            },
        )
    }

    #[test]
    fn emission_materializes_index_growth_at_the_gauges_weight() {
        let (emitter, gauge) = records();
        let receipt = emit_to_gauge(EmitToGaugeArgs {
            emitter: emitter.clone(),
            gauge: gauge.clone(),
            at: 10,
        })
        .unwrap();

        assert_eq!(receipt.emitted, 8); // (10 - 8) index units * 4 votes.
        assert_eq!(
            receipt.emitter_next,
            Emitter {
                total_emitted: 108,
                ..emitter
            }
        );
        assert_eq!(
            receipt.gauge_next,
            Gauge {
                total_emitted: 28,
                emission_index_seen: amount(10),
                last_updated_at_unix_seconds: 10,
                ..gauge
            }
        );
    }

    #[test]
    fn reweighting_settles_the_old_weight_before_replacing_its_contribution() {
        let (emitter, gauge) = records();
        let receipt = set_gauge_votes(SetGaugeVotesArgs {
            emitter: emitter.clone(),
            gauge: gauge.clone(),
            votes: 10,
            at: 20,
        })
        .unwrap();

        // Eight pending units plus 120 units/second * 10 seconds * 4/6.
        assert_eq!(receipt.emitted, 808);
        assert_eq!(
            receipt.emitter_next,
            Emitter {
                total_emitted: 908,
                total_gauge_weight: 12, // Remove four old votes, then add ten.
                emission_index: amount(210),
                last_updated_at_unix_seconds: 20,
                ..emitter
            }
        );
        assert_eq!(
            receipt.gauge_next,
            Gauge {
                votes: 10,
                total_emitted: 828,
                emission_index_seen: amount(210),
                last_updated_at_unix_seconds: 20,
                ..gauge
            }
        );
    }
}

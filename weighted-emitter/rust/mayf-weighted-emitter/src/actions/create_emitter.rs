use core::num::NonZeroU128;

use mayf_fixed_number::{DirectedRounding, FixedNumber, NonNeg};

use crate::{Emitter, Result, Schedule, Seconds, WeightedEmitterError};

pub struct CreateEmitterArgs<I, N: FixedNumber> {
    pub id: I,
    /// Token base units per second at creation.
    pub emission_rate: NonNeg<N>,
    pub annual_decay_rate: NonNeg<N>,
    pub at: Seconds,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateEmitterReceipt<I, N: FixedNumber> {
    pub new_emitter: Emitter<I, N>,
}

/// Create an empty emitter. IDs, ownership and creation uniqueness belong to
/// the host. The index grid must be fine enough for whole u64 voting weights.
pub fn create_emitter<I, N: FixedNumber>(
    args: CreateEmitterArgs<I, N>,
) -> Result<CreateEmitterReceipt<I, N>> {
    let scale = NonZeroU128::new(1 << 64).ok_or(WeightedEmitterError::InsufficientPrecision)?;
    if N::checked_from_ratio_round(1, scale, DirectedRounding::Floor)? == N::ZERO {
        return Err(WeightedEmitterError::InsufficientPrecision);
    }
    N::checked_from_u64(u64::MAX)?;
    Ok(CreateEmitterReceipt {
        new_emitter: Emitter {
            id: args.id,
            schedule: Schedule {
                emission_rate: args.emission_rate,
                annual_decay_rate: args.annual_decay_rate,
            },
            last_updated_at_unix_seconds: args.at,
            total_emitted: 0,
            total_gauge_weight: 0,
            emission_index: NonNeg::new(N::ZERO)?,
        },
    })
}

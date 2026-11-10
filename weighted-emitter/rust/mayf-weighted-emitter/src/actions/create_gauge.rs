use mayf_fixed_number::FixedNumber;

use crate::{Emitter, Gauge, Result, Seconds};

pub struct CreateGaugeArgs<I, N: FixedNumber> {
    pub emitter: Emitter<I, N>,
    pub id: I,
    pub at: Seconds,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateGaugeReceipt<I, N: FixedNumber> {
    pub emitter_next: Emitter<I, N>,
    pub new_gauge: Gauge<I, N>,
}

/// Create a zero-weight gauge at the current index. Past emission is skipped.
pub fn create_gauge<I: Copy, N: FixedNumber>(
    args: CreateGaugeArgs<I, N>,
) -> Result<CreateGaugeReceipt<I, N>> {
    let emitter_next = super::advance(args.emitter, args.at)?;
    let new_gauge = Gauge {
        id: args.id,
        emitter_id: emitter_next.id,
        votes: 0,
        emission_index_seen: emitter_next.emission_index,
        total_emitted: 0,
        last_updated_at_unix_seconds: args.at,
    };
    Ok(CreateGaugeReceipt {
        emitter_next,
        new_gauge,
    })
}

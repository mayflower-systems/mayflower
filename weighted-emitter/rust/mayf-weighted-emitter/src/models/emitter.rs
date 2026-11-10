use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::{Amount, Schedule, Seconds};

/// One schedule and the index that divides its emission across all gauges.
/// The host preserves this canonical record and every receipt atomically.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Emitter<I, N: FixedNumber> {
    pub id: I,
    pub schedule: Schedule<N>,
    pub last_updated_at_unix_seconds: Seconds,
    /// Sum of whole units materialized to gauges; not scheduled or minted supply.
    pub total_emitted: Amount,
    /// Sum of the canonical gauges' votes. Each gauge change updates this sum.
    pub total_gauge_weight: Amount,
    /// Cumulative token base units per vote. Never reset when weights change.
    pub emission_index: NonNeg<N>,
}

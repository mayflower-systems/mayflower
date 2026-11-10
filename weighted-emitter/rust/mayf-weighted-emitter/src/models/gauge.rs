use mayf_fixed_number::{FixedNumber, NonNeg};

use crate::{Amount, Seconds};

/// A gauge's weight, index checkpoint, and cumulative whole-unit allocation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gauge<I, N: FixedNumber> {
    pub id: I,
    pub emitter_id: I,
    pub votes: Amount,
    pub emission_index_seen: NonNeg<N>,
    /// Whole units assigned to this gauge; a downstream consumer can track deltas.
    pub total_emitted: Amount,
    pub last_updated_at_unix_seconds: Seconds,
}

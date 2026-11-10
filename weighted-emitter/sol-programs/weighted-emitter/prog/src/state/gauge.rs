use anchor_lang::prelude::*;
use mayf_weighted_emitter::Gauge as LibGauge;

use super::numbers::{Number, index_from_wire};

#[account]
#[derive(InitSpace)]
pub struct Gauge {
    pub emitter: Pubkey,
    pub total_deposited: u64,
    /// Accrued whole emission, including allocations checkpointed by vote changes.
    pub total_emitted: u64,
    /// The authority may mint the difference between total_emitted and total_minted.
    pub total_minted: u64,
    pub last_updated_at_unix_seconds: u64,
    pub emission_index_seen_q192f64: [u8; 32],
}

impl Gauge {
    pub fn to_library(&self, id: Pubkey) -> LibGauge<Pubkey, Number> {
        LibGauge {
            id,
            emitter_id: self.emitter,
            votes: self.total_deposited,
            emission_index_seen: index_from_wire(self.emission_index_seen_q192f64),
            total_emitted: self.total_emitted,
            last_updated_at_unix_seconds: self.last_updated_at_unix_seconds,
        }
    }

    pub fn apply_library(&mut self, state: LibGauge<Pubkey, Number>) {
        self.total_deposited = state.votes;
        self.total_emitted = state.total_emitted;
        self.last_updated_at_unix_seconds = state.last_updated_at_unix_seconds;
        self.emission_index_seen_q192f64 = state.emission_index_seen.get().to_le_bytes();
    }
}

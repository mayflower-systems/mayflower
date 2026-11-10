use anchor_lang::prelude::*;
use mayf_weighted_emitter::{Emitter as LibEmitter, Schedule};

use super::numbers::{Number, index_from_wire, rate_from_wire, rate_to_wire};

/// Canonical schedule and aggregate allocation state for one independent emitter.
#[account]
#[derive(InitSpace)]
pub struct Emitter {
    pub authority: Pubkey,
    pub vote_mint: Pubkey,
    pub emission_mint: Pubkey,
    pub mint_authority_bump: [u8; 1],
    pub emission_rate_q64f64: u128,
    pub annual_decay_rate_q64f64: u128,
    pub last_updated_at_unix_seconds: u64,
    /// Recorded deposits across gauges, independent of unsolicited escrow transfers.
    pub total_deposited: u64,
    /// Whole emission allocated to gauges, including amounts not yet minted.
    pub total_emitted: u64,
    pub total_minted: u64,
    pub emission_index_q192f64: [u8; 32],
}

impl Emitter {
    pub fn to_library(&self, id: Pubkey) -> LibEmitter<Pubkey, Number> {
        LibEmitter {
            id,
            schedule: Schedule {
                emission_rate: rate_from_wire(self.emission_rate_q64f64),
                annual_decay_rate: rate_from_wire(self.annual_decay_rate_q64f64),
            },
            last_updated_at_unix_seconds: self.last_updated_at_unix_seconds,
            total_emitted: self.total_emitted,
            total_gauge_weight: self.total_deposited,
            emission_index: index_from_wire(self.emission_index_q192f64),
        }
    }

    /// Policy, mint bindings and minted supply remain host-owned fields.
    pub fn apply_library(&mut self, state: LibEmitter<Pubkey, Number>) -> Result<()> {
        self.emission_rate_q64f64 = rate_to_wire(state.schedule.emission_rate)?;
        self.annual_decay_rate_q64f64 = rate_to_wire(state.schedule.annual_decay_rate)?;
        self.last_updated_at_unix_seconds = state.last_updated_at_unix_seconds;
        self.total_deposited = state.total_gauge_weight;
        self.total_emitted = state.total_emitted;
        self.emission_index_q192f64 = state.emission_index.get().to_le_bytes();
        Ok(())
    }
}

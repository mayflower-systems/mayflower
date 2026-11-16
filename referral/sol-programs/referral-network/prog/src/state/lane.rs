use anchor_lang::prelude::*;

/// One permanent asset slot and every field of the kernel's `Lane`/`LanePool`.
#[account]
#[derive(InitSpace)]
pub struct Lane {
    pub network: Pubkey,
    /// Asset identity; eligible balances can represent internal positions.
    pub asset: Pubkey,
    /// Canonical external earnings record. Its adapter must authenticate data
    /// and establish freshness before any eligibility change.
    pub earnings_source: Pubkey,
    pub lane_index: u8,
    pub balance: u64,
    pub reward_index_q192f64: [u8; 32],
    pub remainder_q192f64: [u8; 32],
    pub unallocated_q192f64: [u8; 32],
    /// Consumed earnings, distinct from the source's current cumulative total.
    pub total_earned_seen_q192f64: [u8; 32],
}

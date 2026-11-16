use anchor_lang::prelude::*;

/// A canonical `(network, lane, node)` position. Preserve it after withdrawal
/// so the node's permanent lane mask continues to prove source completeness.
#[account]
#[derive(InitSpace)]
pub struct LaneNode {
    pub network: Pubkey,
    pub lane: Pubkey,
    pub node: Pubkey,
    pub balance: u64,
    pub downline: u64,
    pub reward_index_seen_q192f64: [u8; 32],
    /// Private own-yield carry in micro-basis-point numerator units.
    pub own_carry_q192f64: [u8; 32],
}

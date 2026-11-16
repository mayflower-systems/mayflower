use anchor_lang::prelude::*;

use crate::state::com::Offer;

/// Permanent membership and every field of the kernel's family accounting.
/// The account address supplies the kernel's node identity.
#[account]
#[derive(InitSpace)]
pub struct Node {
    pub network: Pubkey,
    pub owner: Pubkey,
    /// Owner-selected index permits multiple nodes per owner in one network.
    pub node_index: u32,
    /// `None` denotes an earning top-level seat, not the non-earning root.
    pub parent: Option<Pubkey>,
    pub depth: u8,
    /// Every lane ever materialized, including fully withdrawn positions.
    pub lane_mask: u16,
    pub offer: Offer,
    pub sharing_mbps: u32,
    pub kept_q192f64: [u8; 32],
    /// Issued credits held in exclusive custody under the immediate parent.
    pub escrow: u64,
    pub child_escrow: u64,
    pub gift_index_q192f64: [u8; 32],
    pub parent_gift_index_seen_q192f64: [u8; 32],
    /// Private retained-override carry in squared rate-denominator units.
    pub retained_carry_q192f64: [u8; 32],
    /// Shared marketing carry in squared rate-denominator units.
    pub marketing_carry_q192f64: [u8; 32],
    pub gift_remainder_q192f64: [u8; 32],
    pub unallocated_q192f64: [u8; 32],
    /// Closed marketing carry, still in squared rate-denominator units.
    pub unallocated_numerator_q192f64: [u8; 32],
}

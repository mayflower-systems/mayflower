use anchor_lang::prelude::*;

use crate::state::com::NetworkTerms;

/// One authority's network, including the kernel's protocol terms and registry.
#[account]
#[derive(InitSpace)]
pub struct Network {
    /// Authorizes lane registration and eligible balance updates.
    pub authority: Pubkey,
    pub credits_mint: Pubkey,
    /// Immutable after creation; owners cannot change network-wide commissions.
    pub terms: NetworkTerms,
    /// Registered slots are consecutive, bounded by `mayf_referral::MAX_LANES`.
    pub lane_count: u8,
    pub self_bump: [u8; 1],
}

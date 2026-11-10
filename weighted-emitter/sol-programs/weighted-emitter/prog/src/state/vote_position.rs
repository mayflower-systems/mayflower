use anchor_lang::prelude::*;

use crate::constants::VOTE_POSITION_SEED;

#[account]
#[derive(InitSpace)]
pub struct VotePosition {
    pub authority: Pubkey,
    pub gauge: Pubkey,
    pub vote_escrow: Pubkey,
    /// Recorded principal; external token donations do not add voting weight.
    pub deposited: u64,
    pub self_bump: [u8; 1],
}

impl VotePosition {
    pub fn signer_seeds(&self) -> [&[u8]; 4] {
        [
            VOTE_POSITION_SEED,
            self.gauge.as_ref(),
            self.authority.as_ref(),
            self.self_bump.as_ref(),
        ]
    }
}

use anchor_lang::prelude::*;

use crate::constants::TENANT_SEED;

// An immutable numbered tenant within an organization.
#[account]
#[derive(InitSpace)]
pub struct Tenant {
    pub organization: Pubkey,
    pub tenant_index: u32,
    pub clearing_credits_mint: Pubkey,
    pub master_start_time_unix_seconds: u64,
    pub epoch_duration_seconds: u32,

    /// Monotonic epoch index for this tenant.
    pub current_epoch_index: u32,

    /// Creation-time bump used only to sign the permissionless burn.
    pub self_bump: [u8; 1],
}

impl Tenant {
    pub fn signer_seeds<'a>(&'a self, index: &'a [u8; 4]) -> [&'a [u8]; 4] {
        [
            TENANT_SEED,
            self.organization.as_ref(),
            index,
            self.self_bump.as_ref(),
        ]
    }

    pub fn safe_next_epoch_index(&self) -> u32 {
        self.current_epoch_index.checked_add(1).unwrap()
    }
}

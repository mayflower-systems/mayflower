use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Organization {
    pub authority: Pubkey,
    pub next_channel_group_index: u32,
}

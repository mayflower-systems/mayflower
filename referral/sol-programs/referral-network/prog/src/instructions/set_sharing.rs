use anchor_lang::prelude::*;

use crate::{
    error::ReferralNetworkProgramError,
    state::{network::Network, node::Node},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetSharingArgs {
    /// Prospective fraction of gross overrides; 1,000,000 means 100%.
    pub sharing_mbps: u32,
}

#[derive(Accounts)]
pub struct SetSharingAccounts<'info> {
    pub owner: Signer<'info>,

    pub network: Account<'info, Network>,

    #[account(
        mut,
        has_one = network,
        has_one = owner
    )]
    pub node: Box<Account<'info, Node>>,
}

#[inline(never)]
pub fn set_sharing_handler(_ctx: Context<SetSharingAccounts>, _args: SetSharingArgs) -> Result<()> {
    // TODO: Validate the rate and load exactly every permanent source lane and
    // position, with authenticated earnings current through this change. Call
    // set_sharing and persist all records, including closed shared residuals.
    err!(ReferralNetworkProgramError::InstructionNotImplemented)
}

use anchor_lang::prelude::*;

use crate::{
    error::ReferralNetworkProgramError,
    state::{lane::Lane, network::Network, node::Node},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetBalanceArgs {
    /// Target eligible asset balance, not a caller-supplied reward amount.
    pub balance: u64,
}

#[derive(Accounts)]
pub struct SetBalanceAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(has_one = authority)]
    pub network: Account<'info, Network>,

    #[account(
        mut,
        has_one = network
    )]
    pub lane: Account<'info, Lane>,

    #[account(
        mut,
        has_one = network
    )]
    pub node: Box<Account<'info, Node>>,

    pub system_program: Program<'info, System>,
}

#[inline(never)]
pub fn set_balance_handler(_ctx: Context<SetBalanceAccounts>, _args: SetBalanceArgs) -> Result<()> {
    // TODO: Authenticate current earnings and the complete parent-first path;
    // load/materialize canonical lane positions on the heap. Call the kernel's
    // set_balance and atomically persist every returned record with eligibility.
    err!(ReferralNetworkProgramError::InstructionNotImplemented)
}

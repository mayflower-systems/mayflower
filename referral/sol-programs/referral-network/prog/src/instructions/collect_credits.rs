use anchor_lang::prelude::*;

use crate::{
    error::ReferralNetworkProgramError,
    state::{network::Network, node::Node},
};

#[derive(Accounts)]
pub struct CollectCreditsAccounts<'info> {
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
pub fn collect_credits_handler(_ctx: Context<CollectCreditsAccounts>) -> Result<()> {
    // TODO: Add authenticated mint authority and owner destination accounts;
    // call collect_credits and mint only the returned net whole-unit claim,
    // preserving its fraction. Collection does not implicitly settle sources.
    err!(ReferralNetworkProgramError::InstructionNotImplemented)
}

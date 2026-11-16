use anchor_lang::prelude::*;

use crate::{
    error::ReferralNetworkProgramError,
    state::{network::Network, node::Node},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetCreditStakeArgs {
    /// Target escrow principal in already-issued credit base units.
    pub stake: u64,
}

#[derive(Accounts)]
pub struct SetCreditStakeAccounts<'info> {
    pub owner: Signer<'info>,

    pub network: Account<'info, Network>,

    #[account(
        mut,
        has_one = network
    )]
    pub parent: Box<Account<'info, Node>>,

    #[account(
        mut,
        has_one = network,
        has_one = owner,
        constraint = node.parent == Some(parent.key())
            @ ReferralNetworkProgramError::WrongParent
    )]
    pub node: Box<Account<'info, Node>>,
}

#[inline(never)]
pub fn set_credit_stake_handler(
    _ctx: Context<SetCreditStakeAccounts>,
    _args: SetCreditStakeArgs,
) -> Result<()> {
    // TODO: Load every parent source at the same current boundary, then call
    // set_credit_stake. Add exclusive credit escrow accounts and transfer the
    // returned principal delta atomically with all receipt records; never mint it.
    err!(ReferralNetworkProgramError::InstructionNotImplemented)
}

use anchor_lang::prelude::*;

use crate::{
    constants::NODE_SEED,
    error::ReferralNetworkProgramError,
    state::{com::Offer, network::Network, node::Node},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct JoinUnderParentArgs {
    pub node_index: u32,
    pub offer: Offer,
    pub maximum_fee: u64,
}

#[derive(Accounts)]
#[instruction(args: JoinUnderParentArgs)]
pub struct JoinUnderParentAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub owner: Signer<'info>,

    pub network: Account<'info, Network>,

    #[account(
        mut,
        has_one = network
    )]
    pub parent: Box<Account<'info, Node>>,

    #[account(
        init,
        payer = payer,
        space = 8 + Node::INIT_SPACE,
        seeds = [NODE_SEED, network.key().as_ref(), owner.key().as_ref(), args.node_index.to_le_bytes().as_ref()],
        bump
    )]
    pub node: Box<Account<'info, Node>>,

    pub system_program: Program<'info, System>,
}

#[inline(never)]
pub fn join_under_parent_handler(
    _ctx: Context<JoinUnderParentAccounts>,
    _args: JoinUnderParentArgs,
) -> Result<()> {
    // TODO: Add credit custody accounts; call join_under_parent and transfer
    // the receipt's fee to the parent's owner. Persist the child and raised
    // parent offer atomically. The fee never becomes child escrow.
    err!(ReferralNetworkProgramError::InstructionNotImplemented)
}

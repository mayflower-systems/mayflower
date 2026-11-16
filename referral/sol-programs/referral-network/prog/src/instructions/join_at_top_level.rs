use anchor_lang::prelude::*;

use crate::{
    constants::NODE_SEED,
    error::ReferralNetworkProgramError,
    state::{com::Offer, network::Network, node::Node},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct JoinAtTopLevelArgs {
    pub node_index: u32,
    pub offer: Offer,
}

#[derive(Accounts)]
#[instruction(args: JoinAtTopLevelArgs)]
pub struct JoinAtTopLevelAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub owner: Signer<'info>,

    pub network: Account<'info, Network>,

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
pub fn join_at_top_level_handler(
    _ctx: Context<JoinAtTopLevelAccounts>,
    _args: JoinAtTopLevelArgs,
) -> Result<()> {
    // TODO: Authenticate an auction award bound to this network, owner and
    // node. Call join_at_top_level, consume the award exactly once and burn its
    // already-issued credit price atomically with creating the node. The award
    // and token accounts must be added before enabling this instruction.
    err!(ReferralNetworkProgramError::InstructionNotImplemented)
}

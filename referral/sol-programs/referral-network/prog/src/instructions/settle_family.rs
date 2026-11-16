use anchor_lang::prelude::*;

use crate::{
    error::ReferralNetworkProgramError,
    state::{network::Network, node::Node},
};

/// Permissionless settlement of incoming gifts, including for zero-asset nodes.
#[derive(Accounts)]
pub struct SettleFamilyAccounts<'info> {
    pub network: Account<'info, Network>,

    #[account(
        mut,
        has_one = network
    )]
    pub parent: Box<Account<'info, Node>>,

    #[account(
        mut,
        has_one = network,
        constraint = node.parent == Some(parent.key())
            @ ReferralNetworkProgramError::WrongParent
    )]
    pub node: Box<Account<'info, Node>>,
}

#[inline(never)]
pub fn settle_family_handler(_ctx: Context<SettleFamilyAccounts>) -> Result<()> {
    // TODO: Load the parent's complete source set at one authenticated boundary;
    // call settle_family and persist the parent, child and parent positions.
    // Source lanes are projected read-only; received gifts remain terminal.
    err!(ReferralNetworkProgramError::InstructionNotImplemented)
}

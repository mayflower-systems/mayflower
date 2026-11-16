use anchor_lang::prelude::*;

use crate::{
    constants::LANE_NODE_SEED,
    error::ReferralNetworkProgramError,
    state::{lane::Lane, lane_node::LaneNode, network::Network, node::Node},
};

/// Permissionless settlement of an existing position; no owner signature.
#[derive(Accounts)]
pub struct SettleNodeInLaneAccounts<'info> {
    pub network: Account<'info, Network>,

    #[account(has_one = network)]
    pub lane: Account<'info, Lane>,

    #[account(
        mut,
        has_one = network
    )]
    pub node: Box<Account<'info, Node>>,

    #[account(
        mut,
        has_one = network,
        has_one = lane,
        has_one = node,
        seeds = [LANE_NODE_SEED, lane.key().as_ref(), node.key().as_ref()],
        bump
    )]
    pub lane_node: Account<'info, LaneNode>,
}

#[inline(never)]
pub fn settle_node_in_lane_handler(_ctx: Context<SettleNodeInLaneAccounts>) -> Result<()> {
    // TODO: Authenticate current lane earnings and the complete parent-first
    // ancestry/positions. Call settle_node_in_lane and persist its receipt.
    // This action projects the lane read-only; it does not advance its cursor.
    err!(ReferralNetworkProgramError::InstructionNotImplemented)
}

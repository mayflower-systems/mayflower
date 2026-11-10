use alloc::vec::Vec;
use mayf_fixed_number::FixedNumber;

use crate::tree::LaneChain;
use crate::{Identifiable, Lane, LaneNode, Node, Protocol, Result};

pub struct SettleNodeInLaneArgs<I, N> {
    pub protocol: Protocol,
    pub lane: Lane<N>,
    pub node: Node<I, N>,
    pub lineage: Vec<Node<I, N>>,
    pub lane_nodes: Vec<(I, LaneNode<N>)>,
    /// This lane's authenticated cumulative earnings through the settlement.
    pub total_earned: N,
}

pub struct SettleNodeInLaneReceipt<I, N> {
    pub node_next: Node<I, N>,
    pub lineage_next: Vec<Node<I, N>>,
    pub lane_nodes_next: Vec<(I, LaneNode<N>)>,
}

/// Permissionlessly settle one lane and the node's ancestry. Global lane state
/// is projected read-only; only the returned node records are stored. This is
/// a partial settlement, not a promise that every family source is current.
pub fn settle_node_in_lane<I: Identifiable, N: FixedNumber>(
    args: SettleNodeInLaneArgs<I, N>,
) -> Result<SettleNodeInLaneReceipt<I, N>> {
    args.protocol.check_lane(args.lane.id)?;
    let chain = LaneChain::new(
        args.protocol.terms,
        args.lane.id,
        args.node,
        args.lineage,
        args.lane_nodes,
    )?;
    let index = args.lane.reward_index(args.total_earned)?;
    let (node_next, lineage_next, lane_nodes_next) =
        chain.settled(args.protocol.terms, index)?.into_parts();
    Ok(SettleNodeInLaneReceipt {
        node_next,
        lineage_next,
        lane_nodes_next,
    })
}

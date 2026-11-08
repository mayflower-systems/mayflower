use alloc::vec::Vec;

use mayf_fixed_number::FixedNumber;

use crate::tree::LaneChain;
use crate::{Identifiable, Lane, LaneNode, Node, Protocol, Result, Seconds};

pub struct SettleNodeInLaneArgs<I, N> {
    pub protocol: Protocol<N>,
    pub lane: Lane<N>,
    /// The node to settle: for example a member that wants its credits, or a parent whose latest
    /// sharing its children want applied
    pub node: Node<I, N>,
    /// The node's lineage: its parent, its parent's parent, and so on up to a top-level member
    pub lineage: Vec<Node<I, N>>,
    /// The lane nodes the host has in this lane for the node and its lineage, by node id
    pub lane_nodes: Vec<(I, LaneNode<N>)>,
    pub now: Seconds,
}

pub struct SettleNodeInLaneReceipt<I, N> {
    pub node_next: Node<I, N>,
    /// The node's lineage, parent first, with the credits each keeps from this lane
    pub lineage_next: Vec<Node<I, N>>,
    /// The lane nodes to store in this lane, by node id. Settling never adds a new one.
    pub lane_nodes_next: Vec<(I, LaneNode<N>)>,
}

/// Settle a node and its lineage in a lane up to `now`. Each one keeps the credits it earned
/// there in its node, ready to collect, and shares its latest sharing portion there from now on.
///
/// Anyone may call it, on any node. It moves no balance, takes credits from no one, and creates
/// no records, because a node with no lane node here holds nothing here. Settling more often
/// changes results only through rounding, by less than one credit base unit per node per
/// settlement. The lane does not change.
pub fn settle_node_in_lane<I, N>(
    args: SettleNodeInLaneArgs<I, N>,
) -> Result<SettleNodeInLaneReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    let chain = LaneChain::new(args.node, args.lineage, args.lane_nodes)?;

    let terms = args.protocol.terms;
    let gauge = args.protocol.gauge_at(args.now)?;
    let reward_index = args.lane.reward_index(&gauge)?;

    let (node_next, lineage_next, lane_nodes_next) =
        chain.settled(terms, reward_index)?.into_parts();

    Ok(SettleNodeInLaneReceipt {
        node_next,
        lineage_next,
        lane_nodes_next,
    })
}

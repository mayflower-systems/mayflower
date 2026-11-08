use alloc::vec::Vec;

use mayf_fixed_number::FixedNumber;

use crate::tree::LaneChain;
use crate::{Amount, Identifiable, Lane, LaneNode, Node, Protocol, Result, Seconds};

pub struct DepositIntoLaneArgs<I, N> {
    pub protocol: Protocol<N>,
    pub lane: Lane<N>,
    /// The depositing member
    pub member: Node<I, N>,
    /// The member's lineage: its parent, its parent's parent, and so on up to a top-level member
    pub lineage: Vec<Node<I, N>>,
    /// The lane nodes the host has in this lane for the member and its lineage, by node id
    pub lane_nodes: Vec<(I, LaneNode<N>)>,
    pub amount: Amount,
    pub now: Seconds,
}

pub struct DepositIntoLaneReceipt<I, N> {
    pub lane_next: Lane<N>,
    pub member_next: Node<I, N>,
    /// The member's lineage, parent first, with the credits each keeps from this lane
    pub lineage_next: Vec<Node<I, N>>,
    /// Every lane node of the member and its lineage in this lane, by node id
    pub lane_nodes_next: Vec<(I, LaneNode<N>)>,
}

/// Add to a member's balance in a lane. The member and its upline earn at the old balance up
/// to `now`.
pub fn deposit_into_lane<I, N>(
    args: DepositIntoLaneArgs<I, N>,
) -> Result<DepositIntoLaneReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    let chain = LaneChain::new(args.member, args.lineage, args.lane_nodes)?;

    let terms = args.protocol.terms;
    let gauge = args.protocol.gauge_at(args.now)?;
    let (lane, reward_index) = args.lane.settle(&gauge)?;

    let (member_next, lineage_next, lane_nodes_next) = chain
        .deposit(terms, reward_index, args.amount)?
        .into_parts();
    let lane_next = Lane {
        pool: lane.pool.deposit(args.amount),
        ..lane
    };

    Ok(DepositIntoLaneReceipt {
        lane_next,
        member_next,
        lineage_next,
        lane_nodes_next,
    })
}

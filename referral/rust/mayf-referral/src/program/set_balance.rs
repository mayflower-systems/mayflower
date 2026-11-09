use alloc::vec::Vec;
use mayf_fixed_number::FixedNumber;

use crate::tree::LaneChain;
use crate::{Amount, Change, Emission, Identifiable, Lane, LaneNode, Node, Protocol, Result};

pub struct SetBalanceArgs<I, N> {
    pub protocol: Protocol<N>,
    pub lane: Lane<N>,
    pub member: Node<I, N>,
    /// Parent first, ending at the top-level seat, bounded by the configured depth.
    pub lineage: Vec<Node<I, N>>,
    /// Existing positions in this lane, keyed by node ID. Omission is rejected
    /// whenever the corresponding node's permanent lane mask has the bit set.
    pub lane_nodes: Vec<(I, LaneNode<N>)>,
    pub balance: Amount,
    pub emission: Emission<N>,
}

pub struct SetBalanceReceipt<I, N> {
    pub protocol_next: Protocol<N>,
    pub lane_next: Lane<N>,
    pub member_next: Node<I, N>,
    pub lineage_next: Vec<Node<I, N>>,
    pub lane_nodes_next: Vec<(I, LaneNode<N>)>,
    /// Asset custody or the host's asset position must change atomically with this receipt.
    pub balance_change: Change,
}

/// Reconcile one asset balance. Settle this lane and its complete ancestry at
/// old balances, then adjust the lane total and each ancestor's strict downline.
/// No siblings, descendants, or other lanes are loaded.
pub fn set_balance<I: Identifiable, N: FixedNumber>(
    mut args: SetBalanceArgs<I, N>,
) -> Result<SetBalanceReceipt<I, N>> {
    args.protocol.check_lane(args.lane.id)?;
    let chain = LaneChain::new(
        args.protocol.terms,
        args.lane.id,
        args.member,
        args.lineage,
        args.lane_nodes,
    )?;
    let change = Change::between(chain.balance(), args.balance);
    let mut gauge = args.protocol.gauge_at(args.emission)?;
    let (mut lane, index) = args.lane.settle(&gauge)?;
    let (member_next, lineage_next, lane_nodes_next) = chain
        .settled(args.protocol.terms, index)?
        .change_balance(change, index)?
        .into_parts();
    lane.pool = match change {
        Change::Add(amount) => lane.pool.deposit(amount)?,
        Change::Remove(amount) => lane.pool.withdraw(amount)?,
    };
    if !matches!(change, Change::Add(0) | Change::Remove(0)) {
        gauge.close_eligibility()?;
    }
    args.protocol.gauge = gauge;
    Ok(SetBalanceReceipt {
        protocol_next: args.protocol,
        lane_next: lane,
        member_next,
        lineage_next,
        lane_nodes_next,
        balance_change: change,
    })
}

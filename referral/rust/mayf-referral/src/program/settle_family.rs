use alloc::vec::Vec;
use mayf_fixed_number::FixedNumber;

use crate::tree::settle_sources;
use crate::{Emission, Error, Identifiable, Lane, LaneId, LaneNode, Node, Protocol, Result};

pub struct SettleFamilyArgs<'a, I, N> {
    pub protocol: &'a Protocol<N>,
    pub parent: Node<I, N>,
    pub child: Node<I, N>,
    pub lanes: &'a [Lane<N>],
    pub parent_lane_nodes: Vec<(LaneId, LaneNode<N>)>,
    pub emission: Emission<N>,
}

pub struct SettleFamilyReceipt<I, N> {
    pub parent_next: Node<I, N>,
    pub child_next: Node<I, N>,
    pub parent_lane_nodes_next: Vec<(LaneId, LaneNode<N>)>,
}

/// Bring the child's incoming gifts current through the snapshot, even when the
/// child has no asset positions. Gifts are terminal, so no grandparent or sibling
/// is needed. Follow with `collect_credits` to mint the accrued net entitlement.
pub fn settle_family<I: Identifiable, N: FixedNumber>(
    mut args: SettleFamilyArgs<'_, I, N>,
) -> Result<SettleFamilyReceipt<I, N>> {
    if args.child.parent_id != Some(args.parent.id) || args.child.id == args.parent.id {
        return Err(Error::WrongParent);
    }
    settle_sources(
        args.protocol,
        &mut args.parent,
        args.lanes,
        &mut args.parent_lane_nodes,
        args.emission,
    )?;
    args.child.receive_gift(args.parent.gift_index)?;
    Ok(SettleFamilyReceipt {
        parent_next: args.parent,
        child_next: args.child,
        parent_lane_nodes_next: args.parent_lane_nodes,
    })
}

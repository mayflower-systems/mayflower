use alloc::vec::Vec;
use mayf_fixed_number::FixedNumber;

use crate::tree::{close_sources, settle_sources};
use crate::{
    Emission, Identifiable, Lane, LaneId, LaneNode, MicroBasisPoints, Node, Protocol, Result,
};

pub struct SetSharingArgs<I, N> {
    pub protocol: Protocol<N>,
    pub member: Node<I, N>,
    /// Global lane records, exactly matching the member's lane mask.
    pub lanes: Vec<Lane<N>>,
    /// The member's positions in those lanes. Returned in the same allocation.
    pub lane_nodes: Vec<(LaneId, LaneNode<N>)>,
    pub sharing: MicroBasisPoints,
    pub emission: Emission<N>,
}

pub struct SetSharingReceipt<I, N> {
    pub protocol_next: Protocol<N>,
    pub lanes_next: Vec<Lane<N>>,
    pub member_next: Node<I, N>,
    pub lane_nodes_next: Vec<(LaneId, LaneNode<N>)>,
}

/// Change the marketing portion of gross overrides prospectively. Every source
/// lane first earns under the old rate and old escrow total, including idle lanes.
pub fn set_sharing<I: Identifiable, N: FixedNumber>(
    mut args: SetSharingArgs<I, N>,
) -> Result<SetSharingReceipt<I, N>> {
    settle_sources(
        &args.protocol,
        &mut args.member,
        &args.lanes,
        &mut args.lane_nodes,
        args.emission,
    )?;
    if args.member.sharing != args.sharing {
        close_sources(&mut args.protocol, &mut args.lanes, args.emission)?;
        args.member.close_family()?;
        args.member.sharing = args.sharing;
    }
    Ok(SetSharingReceipt {
        protocol_next: args.protocol,
        lanes_next: args.lanes,
        member_next: args.member,
        lane_nodes_next: args.lane_nodes,
    })
}

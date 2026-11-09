use alloc::vec::Vec;
use mayf_fixed_number::FixedNumber;

use crate::tree::{close_sources, settle_sources};
use crate::{
    Amount, Change, Emission, Error, Identifiable, Lane, LaneId, LaneNode, Node, Protocol, Result,
};

pub struct SetCreditStakeArgs<I, N> {
    pub protocol: Protocol<N>,
    pub parent: Node<I, N>,
    pub child: Node<I, N>,
    pub lanes: Vec<Lane<N>>,
    pub parent_lane_nodes: Vec<(LaneId, LaneNode<N>)>,
    /// Target principal, in already-issued credit base units.
    pub stake: Amount,
    pub emission: Emission<N>,
}

pub struct SetCreditStakeReceipt<I, N> {
    pub protocol_next: Protocol<N>,
    pub lanes_next: Vec<Lane<N>>,
    pub parent_next: Node<I, N>,
    pub child_next: Node<I, N>,
    pub parent_lane_nodes_next: Vec<(LaneId, LaneNode<N>)>,
    /// Add transfers credits into this child's escrow; Remove returns principal.
    /// Neither mints credits. Custody must exclude use in any other program.
    pub escrow_change: Change,
}

/// Settle every parent source and the child's old entitlement before changing
/// principal or the denominator. Full withdrawal is allowed; rewards stay liquid
/// and never auto-compound. A zero-asset child can stake and earn.
pub fn set_credit_stake<I: Identifiable, N: FixedNumber>(
    mut args: SetCreditStakeArgs<I, N>,
) -> Result<SetCreditStakeReceipt<I, N>> {
    if args.child.parent_id != Some(args.parent.id) || args.child.id == args.parent.id {
        return Err(Error::WrongParent);
    }
    settle_sources(
        &args.protocol,
        &mut args.parent,
        &args.lanes,
        &mut args.parent_lane_nodes,
        args.emission,
    )?;
    args.child.receive_gift(args.parent.gift_index)?;
    let change = Change::between(args.child.escrow, args.stake);
    if args.child.escrow != args.stake {
        close_sources(&mut args.protocol, &mut args.lanes, args.emission)?;
        args.parent.close_family()?;
        args.parent.child_escrow = change.apply(args.parent.child_escrow)?;
        args.child.escrow = args.stake;
    }
    Ok(SetCreditStakeReceipt {
        protocol_next: args.protocol,
        lanes_next: args.lanes,
        parent_next: args.parent,
        child_next: args.child,
        parent_lane_nodes_next: args.parent_lane_nodes,
        escrow_change: change,
    })
}

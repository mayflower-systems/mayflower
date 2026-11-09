use mayf_fixed_number::FixedNumber;

use crate::{Amount, Error, Identifiable, Node, Offer, Protocol, Result};

pub struct JoinUnderParentArgs<I, N> {
    pub protocol: Protocol<N>,
    pub parent: Node<I, N>,
    pub new_node_id: I,
    pub new_node_offer: Offer,
    pub maximum_fee: Amount,
}

pub struct JoinUnderParentReceipt<I, N> {
    pub parent_next: Node<I, N>,
    pub new_node: Node<I, N>,
    /// Transfer issued credits from the newcomer to the parent's owner.
    pub join_fee: Amount,
}

/// Permanently join at the current advertised fee, then saturating-increment it.
/// Joining creates zero escrow, so it needs no lane settlement. The host ensures
/// the new ID is unused; a later first stake performs the complete parent barrier.
pub fn join_under_parent<I: Identifiable, N: FixedNumber>(
    args: JoinUnderParentArgs<I, N>,
) -> Result<JoinUnderParentReceipt<I, N>> {
    if args.parent.depth >= args.protocol.terms.max_depth() {
        return Err(Error::ParentAtMaxDepth);
    }
    if args.new_node_id == args.parent.id {
        return Err(Error::BrokenLineage);
    }
    if args.parent.offer.current_fee > args.maximum_fee {
        return Err(Error::JoinFeeExceeded);
    }
    let (parent_next, new_node, join_fee) =
        args.parent.admit(args.new_node_id, args.new_node_offer)?;
    Ok(JoinUnderParentReceipt {
        parent_next,
        new_node,
        join_fee,
    })
}

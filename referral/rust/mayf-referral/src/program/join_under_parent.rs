use alloc::vec::Vec;

use mayf_fixed_number::FixedNumber;

use crate::{Amount, Error, Identifiable, Node, Offer, Protocol, Result};

pub struct JoinUnderParentArgs<I, N> {
    pub protocol: Protocol<N>,
    /// The node to join directly under
    pub parent: Node<I, N>,
    /// The parent's lineage: its parent, its parent's parent, and so on up to a top-level member
    pub lineage: Vec<Node<I, N>>,
    pub new_node_id: I,
    /// The new node's price for its own children
    pub new_node_offer: Offer,
}

pub struct JoinUnderParentReceipt<I, N> {
    pub parent_next: Node<I, N>,
    pub new_node: Node<I, N>,
    /// The fee the host moves from the new member to the parent's owner
    pub join_fee: Amount,
}

/// Admit a new member directly under a parent, at the parent's current fee. The parent's fee
/// then rises by its increment.
pub fn join_under_parent<I, N>(
    args: JoinUnderParentArgs<I, N>,
) -> Result<JoinUnderParentReceipt<I, N>>
where
    I: Identifiable,
    N: FixedNumber,
{
    args.parent.validate_lineage(&args.lineage)?;

    // The parent's depth is the length of its lineage, and the new member sits one below it.
    if args.lineage.len() >= usize::from(args.protocol.terms.max_depth()) {
        return Err(Error::ParentAtMaxDepth);
    }

    let (parent_next, new_node, join_fee) =
        args.parent.admit(args.new_node_id, args.new_node_offer);

    Ok(JoinUnderParentReceipt {
        parent_next,
        new_node,
        join_fee,
    })
}

use mayf_fixed_number::FixedNumber;

use crate::{Amount, Identifiable, Node, Offer, Protocol};

pub struct JoinAtTopLevelArgs<I, N> {
    pub protocol: Protocol<N>,
    pub new_node_id: I,
    /// The new node's price for its own children
    pub new_node_offer: Offer,
}

pub struct JoinAtTopLevelReceipt<I, N> {
    pub protocol_next: Protocol<N>,
    pub new_node: Node<I, N>,
    /// The fee the host moves from the new member to the protocol
    pub join_fee: Amount,
}

/// Admit a new member to a top-level seat, under no parent, at the current top-level fee. The
/// top-level fee then rises by its increment.
pub fn join_at_top_level<I, N>(args: JoinAtTopLevelArgs<I, N>) -> JoinAtTopLevelReceipt<I, N>
where
    I: Identifiable,
    N: FixedNumber,
{
    let protocol = args.protocol;
    let (top_level_offer, join_fee) = protocol.top_level_offer.charge();

    JoinAtTopLevelReceipt {
        protocol_next: Protocol {
            top_level_offer,
            ..protocol
        },
        new_node: Node::top_level(args.new_node_id, args.new_node_offer),
        join_fee,
    }
}

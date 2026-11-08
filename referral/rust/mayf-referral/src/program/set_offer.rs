use mayf_fixed_number::FixedNumber;

use crate::{Identifiable, Node, Offer};

pub struct SetOfferArgs<I, N> {
    pub node: Node<I, N>,
    pub offer: Offer,
}

pub struct SetOfferReceipt<I, N> {
    pub node_next: Node<I, N>,
}

/// Set a node's price for new children. The next child pays the new current fee.
pub fn set_offer<I, N>(args: SetOfferArgs<I, N>) -> SetOfferReceipt<I, N>
where
    I: Identifiable,
    N: FixedNumber,
{
    let node_next = Node {
        offer: args.offer,
        ..args.node
    };

    SetOfferReceipt { node_next }
}

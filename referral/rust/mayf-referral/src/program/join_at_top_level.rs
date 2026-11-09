use mayf_fixed_number::FixedNumber;

use crate::{Amount, Error, Identifiable, Node, Offer, Result, RootSeat};

pub struct JoinAtTopLevelArgs<I> {
    pub seat: RootSeat<I>,
    pub new_node_offer: Offer,
}

pub struct JoinAtTopLevelReceipt<I, N> {
    pub seat_next: RootSeat<I>,
    pub new_node: Node<I, N>,
    /// Burn already-issued credits, atomically with activation. Never mint them first.
    pub credits_to_burn: Amount,
}

/// Activate one authenticated auction award under the non-earning root.
/// The host enforces the auction's scarcity and persists `seat_next`, preventing replay.
pub fn join_at_top_level<I: Identifiable, N: FixedNumber>(
    args: JoinAtTopLevelArgs<I>,
) -> Result<JoinAtTopLevelReceipt<I, N>> {
    let RootSeat::Awarded { node_id, price } = args.seat else {
        return Err(Error::SeatAlreadyActivated);
    };
    Ok(JoinAtTopLevelReceipt {
        seat_next: RootSeat::Activated,
        new_node: Node::top_level(node_id, args.new_node_offer),
        credits_to_burn: price,
    })
}

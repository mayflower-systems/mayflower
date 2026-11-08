use mayf_fixed_number::FixedNumber;

use crate::{Identifiable, MicroBasisPoints, Node};

pub struct SetSharingArgs<I, N> {
    pub member: Node<I, N>,
    /// The portion of the credits it earns, in every lane, that the member shares with its
    /// direct children
    pub sharing: MicroBasisPoints,
}

pub struct SetSharingReceipt<I, N> {
    pub member_next: Node<I, N>,
}

/// Set the portion of its credits a member shares with its direct children, in every lane.
///
/// Each lane applies the new portion from its next settlement, and until then uses the portion
/// it last settled with. So a change never reaches back into credits already earned. A deposit
/// or withdrawal by the member or any of its descendants settles the member in that lane, and
/// anyone can settle it there with [`settle_node_in_lane`](super::settle_node_in_lane).
pub fn set_sharing<I, N>(args: SetSharingArgs<I, N>) -> SetSharingReceipt<I, N>
where
    I: Identifiable,
    N: FixedNumber,
{
    let member_next = Node {
        sharing: args.sharing,
        ..args.member
    };

    SetSharingReceipt { member_next }
}

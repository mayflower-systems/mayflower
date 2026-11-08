use mayf_fixed_number::{FixedNumber, IntegralRounding, MathResult};
use mayf_micro_basis_points::MicroBasisPoints;

use super::{Identifiable, Offer};
use crate::{Amount, Error, Result};

/// A member of the referral tree, and what belongs to it in every lane: its place in the tree,
/// its offer, its sharing, and the credits it keeps. Nodes never move.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node<I, N> {
    pub id: I,
    /// The node this one joined directly under. A top-level member has none.
    pub parent_id: Option<I>,
    /// The price for new children to join directly under this node
    pub offer: Offer,
    /// The portion of the credits it earns, in every lane, that the member shares with its
    /// direct children. Each lane applies it from that lane's next settlement, and until then
    /// uses its lane node's [`sharing_seen`](super::LaneNode::sharing_seen).
    pub sharing: MicroBasisPoints,
    /// Credits the member keeps and has not collected, from every lane
    pub kept: N,
}

impl<I: Identifiable, N: FixedNumber> Node<I, N> {
    /// A member that took a top-level seat, under no parent
    pub(crate) fn top_level(id: I, offer: Offer) -> Self {
        Self::joined(id, None, offer)
    }

    /// Check that `lineage` is this node's lineage: its parent, its parent's parent, and so on up
    /// to a top-level member. A top-level member's lineage is empty.
    pub(crate) fn validate_lineage(&self, lineage: &[Node<I, N>]) -> Result<()> {
        let mut parent_id = self.parent_id;

        for node in lineage {
            if parent_id != Some(node.id) {
                return Err(Error::BrokenLineage);
            }
            parent_id = node.parent_id;
        }

        if parent_id.is_some() {
            return Err(Error::BrokenLineage);
        }

        Ok(())
    }

    /// Admit a new child at this node's current fee, then raise the fee.
    /// Returns this node, the child, and the fee the child pays.
    pub(crate) fn admit(self, id: I, offer: Offer) -> (Self, Self, Amount) {
        let child = Self::joined(id, Some(self.id), offer);
        let (raised, fee) = self.offer.charge();
        let parent = Self {
            offer: raised,
            ..self
        };

        (parent, child, fee)
    }

    /// Take the whole credits the member keeps. The fraction of a base unit stays.
    pub(crate) fn collect(mut self) -> MathResult<(Self, Amount)>
    where
        N: IntegralRounding,
    {
        let whole = self.kept.checked_floor_to_u64()?;
        self.kept = self.kept.checked_sub(N::checked_from_u64(whole)?)?;
        Ok((self, whole))
    }

    /// A new member, which shares nothing and keeps nothing yet
    fn joined(id: I, parent_id: Option<I>, offer: Offer) -> Self {
        Self {
            id,
            parent_id,
            offer,
            sharing: MicroBasisPoints::ZERO,
            kept: N::ZERO,
        }
    }
}

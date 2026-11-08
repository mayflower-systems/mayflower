use mayf_fixed_number::DirectedRounding::Floor;
use mayf_fixed_number::{FixedNumber, MathResult};
use mayf_micro_basis_points::MicroBasisPoints;

use super::NetworkTerms;
use crate::Amount;
use crate::numbers::Change;

/// One node's balances in one lane, and the indexes that turn them into credits.
///
/// The host stores one per lane and node once an action first returns it, which happens once the
/// node holds a balance there. The host never deletes one, because the node's children remember
/// its gift index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneNode<N> {
    /// The node's own balance in the lane
    pub balance: Amount,
    /// The balances of the node's whole downline
    pub downline: Amount,
    /// The balances of the node's direct children
    pub children: Amount,
    /// The lane's reward index when the node last settled
    pub reward_index_seen: N,
    /// The parent's gift index when the node last settled
    pub parent_gift_index_seen: N,
    /// Credits the node has shared per unit of its children's balance
    pub gift_index: N,
    /// The member's sharing when this lane node last settled, which applies until it settles
    /// again. So a change to the member's sharing reaches this lane at its next settlement.
    pub sharing_seen: MicroBasisPoints,
}

/// A lane node the host has none for yet
impl<N: FixedNumber> Default for LaneNode<N> {
    fn default() -> Self {
        Self {
            balance: 0,
            downline: 0,
            children: 0,
            reward_index_seen: N::ZERO,
            parent_gift_index_seen: N::ZERO,
            gift_index: N::ZERO,
            sharing_seen: MicroBasisPoints::ZERO,
        }
    }
}

impl<N: FixedNumber> LaneNode<N> {
    /// Earn up to `reward_index`, after the parent settled at `parent_gift_index`. Returns the
    /// credits the member keeps from this lane.
    ///
    /// The node earns its core points times the reward index's growth. It shares the portion it
    /// last settled with of those earned credits among its children, through its own gift index,
    /// and keeps the rest, including any amount too small to share. It also receives its balance
    /// times the parent gift index's growth, and keeps those gifts whole: gifts never pass further
    /// down. From now on the node shares `sharing`, the member's current portion.
    pub(super) fn settle(
        &mut self,
        terms: NetworkTerms,
        depth: usize,
        reward_index: N,
        parent_gift_index: N,
        sharing: MicroBasisPoints,
    ) -> MathResult<N> {
        let core_points: N = terms.core_points(depth, self.balance, self.downline)?;
        let reward_growth = reward_index.checked_sub(self.reward_index_seen)?;
        let gift_growth = parent_gift_index.checked_sub(self.parent_gift_index_seen)?;
        let earned = core_points.checked_mul_floor(reward_growth)?;
        let gifts = gift_growth.checked_mul_floor(N::checked_from_u64(self.balance)?)?;

        let shared = if self.children == 0 {
            N::ZERO
        } else {
            let children = N::checked_from_u64(self.children)?;
            let portion: N = self.sharing_seen.to_fixed(Floor)?;
            let per_unit = earned.checked_mul_div_floor(portion, children)?;
            self.gift_index = self.gift_index.checked_add(per_unit)?;
            per_unit.checked_mul_floor(children)?
        };

        self.reward_index_seen = reward_index;
        self.parent_gift_index_seen = parent_gift_index;
        self.sharing_seen = sharing;
        earned.checked_sub(shared)?.checked_add(gifts)
    }

    /// Whether the lane node holds nothing: no balance, no downline, no children, and no gifts
    /// shared yet. It then acts exactly like a missing one, because it earns nothing and no child
    /// remembers its gift index.
    pub(super) fn holds_nothing(&self) -> bool {
        self.balance == 0 && self.downline == 0 && self.children == 0 && self.gift_index == N::ZERO
    }

    pub(super) fn change_balance(&mut self, change: Change) {
        self.balance = change.apply(self.balance);
    }

    /// Apply a change to one descendant's balance.
    pub(super) fn change_downline(&mut self, change: Change) {
        self.downline = change.apply(self.downline);
    }

    /// Apply a change to one direct child's balance.
    pub(super) fn change_children(&mut self, change: Change) {
        self.children = change.apply(self.children);
    }
}

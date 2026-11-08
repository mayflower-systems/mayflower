use alloc::vec::Vec;
use core::iter;

use mayf_fixed_number::{FixedNumber, MathResult};

use super::{Identifiable, LaneNode, NetworkTerms, Node};
use crate::numbers::Change;
use crate::{Amount, Error, Result};

const HOLDS_MEMBER: &str = "a chain holds its member";

/// A member and its lineage in one lane: each node with its lane node. What a lane action
/// settles.
///
/// Every action on a member's lane settles this chain and nothing else. A member at the
/// network's maximum depth has the longest lineage, so a chain holds at most that depth plus one
/// nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LaneChain<I, N> {
    /// From the top-level member down to the member, so each link's index is its depth. It is
    /// never empty.
    links: Vec<Link<I, N>>,
}

/// A node in a chain, with its lane node, and whether the host had that lane node
#[derive(Clone, Debug, PartialEq, Eq)]
struct Link<I, N> {
    node: Node<I, N>,
    lane_node: LaneNode<N>,
    stored: bool,
}

/// A settled chain, taken apart for the receipt: the member, its lineage, parent first, and the
/// lane nodes to store, by node id
pub(crate) type ChainParts<I, N> = (Node<I, N>, Vec<Node<I, N>>, Vec<(I, LaneNode<N>)>);

impl<I: Identifiable, N: FixedNumber> LaneChain<I, N> {
    /// Pair `member` and its `lineage` with their lane nodes. `lane_nodes` holds the ones the host
    /// has, by node id. A node the host has none for starts with an empty lane node.
    pub(crate) fn new(
        member: Node<I, N>,
        lineage: Vec<Node<I, N>>,
        mut lane_nodes: Vec<(I, LaneNode<N>)>,
    ) -> Result<Self> {
        member.validate_lineage(&lineage)?;

        let mut take = |id: I| match lane_nodes.iter().position(|(held, _)| *held == id) {
            Some(index) => (lane_nodes.swap_remove(index).1, true),
            None => (LaneNode::default(), false),
        };
        let links = lineage
            .into_iter()
            .rev()
            .chain(iter::once(member))
            .map(|node| {
                let (lane_node, stored) = take(node.id);
                Link {
                    node,
                    lane_node,
                    stored,
                }
            })
            .collect();

        // Any lane node left over belongs to no node here, or repeats one.
        if !lane_nodes.is_empty() {
            return Err(Error::StrayLaneNode);
        }

        Ok(Self { links })
    }

    /// The member's own lane node
    pub(crate) fn member(&self) -> &LaneNode<N> {
        &self.links.last().expect(HOLDS_MEMBER).lane_node
    }

    /// Settle every node at `reward_index`, top-level member first, so each one sees its
    /// parent's settled gift index. Each member keeps its credits in its node.
    pub(crate) fn settled(mut self, terms: NetworkTerms, reward_index: N) -> MathResult<Self> {
        let mut parent_gift_index = N::ZERO;

        for (depth, link) in self.links.iter_mut().enumerate() {
            let (lane_node, sharing) = (&mut link.lane_node, link.node.sharing);
            let kept = lane_node.settle(terms, depth, reward_index, parent_gift_index, sharing)?;
            link.node.kept = link.node.kept.checked_add(kept)?;
            parent_gift_index = lane_node.gift_index;
        }

        Ok(self)
    }

    /// Settle at `reward_index`, then add `amount` to the member's balance.
    pub(crate) fn deposit(
        self,
        terms: NetworkTerms,
        reward_index: N,
        amount: Amount,
    ) -> MathResult<Self> {
        let mut chain = self.settled(terms, reward_index)?;
        chain.change_balance(Change::Add(amount));
        Ok(chain)
    }

    /// Settle at `reward_index`, then remove `amount` from the member's balance, which holds it.
    pub(crate) fn withdraw(
        self,
        terms: NetworkTerms,
        reward_index: N,
        amount: Amount,
    ) -> MathResult<Self> {
        let mut chain = self.settled(terms, reward_index)?;
        chain.change_balance(Change::Remove(amount));
        Ok(chain)
    }

    /// Take the chain apart for the receipt. It leaves out a lane node the host had none for while
    /// that lane node still holds nothing, so an action never creates an empty record.
    pub(crate) fn into_parts(self) -> ChainParts<I, N> {
        let mut nodes = Vec::with_capacity(self.links.len());
        let mut lane_nodes = Vec::with_capacity(self.links.len());

        for link in self.links {
            if link.stored || !link.lane_node.holds_nothing() {
                lane_nodes.push((link.node.id, link.lane_node));
            }
            nodes.push(link.node);
        }

        let member = nodes.pop().expect(HOLDS_MEMBER);
        nodes.reverse();
        (member, nodes, lane_nodes)
    }

    /// Change the member's balance, and each total that includes it: every upline member's
    /// downline, and the parent's children.
    fn change_balance(&mut self, change: Change) {
        let (member, upline) = self.links.split_last_mut().expect(HOLDS_MEMBER);
        member.lane_node.change_balance(change);

        for link in upline.iter_mut() {
            link.lane_node.change_downline(change);
        }

        if let Some(parent) = upline.last_mut() {
            parent.lane_node.change_children(change);
        }
    }
}

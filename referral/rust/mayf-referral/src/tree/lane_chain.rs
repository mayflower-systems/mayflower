use alloc::vec::Vec;
use core::iter;

use mayf_fixed_number::FixedNumber;

use super::{Identifiable, LaneNode, NetworkTerms, Node};
use crate::numbers::Change;
use crate::{Error, LaneId, Result};

/// The original input buffers are also the output buffers. The path is never
/// copied into an inline array or a second vector.
pub(crate) struct LaneChain<I, N> {
    lane: LaneId,
    member: Node<I, N>,
    lineage: Vec<Node<I, N>>,
    lane_nodes: Vec<(I, LaneNode<N>)>,
}

pub(crate) type ChainParts<I, N> = (Node<I, N>, Vec<Node<I, N>>, Vec<(I, LaneNode<N>)>);

impl<I: Identifiable, N: FixedNumber> LaneChain<I, N> {
    pub(crate) fn new(
        terms: NetworkTerms,
        lane: LaneId,
        member: Node<I, N>,
        lineage: Vec<Node<I, N>>,
        lane_nodes: Vec<(I, LaneNode<N>)>,
    ) -> Result<Self> {
        member.validate_lineage(terms.max_depth(), &lineage)?;
        if lane_nodes.len() > lineage.len() + 1 {
            return Err(Error::StrayLaneNode);
        }
        for (index, (id, _)) in lane_nodes.iter().enumerate() {
            if lane_nodes[..index].iter().any(|(other, _)| other == id)
                || !iter::once(&member)
                    .chain(lineage.iter())
                    .any(|node| node.id == *id && node.lanes.contains(lane))
            {
                return Err(Error::StrayLaneNode);
            }
        }
        for node in iter::once(&member).chain(lineage.iter()) {
            if node.lanes.contains(lane) && !lane_nodes.iter().any(|(id, _)| *id == node.id) {
                return Err(Error::MissingLaneNode);
            }
        }
        Ok(Self {
            lane,
            member,
            lineage,
            lane_nodes,
        })
    }

    pub(crate) fn balance(&self) -> u64 {
        self.lane_nodes
            .iter()
            .find(|(id, _)| *id == self.member.id)
            .map_or(0, |(_, state)| state.balance)
    }

    /// Settle sources and terminal incoming gifts, top ancestor first. This
    /// settles only this lane; it never changes the family's sharing terms.
    pub(crate) fn settled(mut self, terms: NetworkTerms, reward_index: N) -> Result<Self> {
        let mut parent_index = N::ZERO;
        for node in self
            .lineage
            .iter_mut()
            .rev()
            .chain(iter::once(&mut self.member))
        {
            if let Some((_, state)) = self.lane_nodes.iter_mut().find(|(id, _)| *id == node.id) {
                let (own, overrides) = state.settle(terms, node.depth, reward_index)?;
                node.earn(own, overrides)?;
            }
            node.receive_gift(parent_index)?;
            parent_index = node.gift_index;
        }
        Ok(self)
    }

    /// Every prior balance earned through this index before any total changes.
    /// A new position starts at the current index, with no historical claim.
    pub(crate) fn change_balance(mut self, change: Change, reward_index: N) -> Result<Self> {
        if matches!(change, Change::Add(0) | Change::Remove(0)) {
            return Ok(self);
        }
        self.lane_nodes
            .reserve(self.lineage.len() + 1 - self.lane_nodes.len());
        let member_id = self.member.id;
        for node in iter::once(&mut self.member).chain(self.lineage.iter_mut()) {
            let index = match self.lane_nodes.iter().position(|(id, _)| *id == node.id) {
                Some(index) => index,
                None => {
                    self.lane_nodes.push((node.id, LaneNode::at(reward_index)));
                    node.lanes.insert(self.lane);
                    self.lane_nodes.len() - 1
                }
            };
            let state = &mut self.lane_nodes[index].1;
            if node.id == member_id {
                state.change_balance(change)?;
            } else {
                state.change_downline(change)?;
            }
        }
        Ok(self)
    }

    pub(crate) fn into_parts(self) -> ChainParts<I, N> {
        (self.member, self.lineage, self.lane_nodes)
    }
}

use mayf_fixed_number::FixedNumber;

use crate::{Error, Identifiable, Lane, LaneId, LaneMask, LaneNode, Node, Protocol, Result};

/// A complete source barrier: settle every lane ever materialized for this
/// parent through the host's current earnings for each lane. This helper
/// projects lanes without writing them; an eligibility change subsequently
/// commits and closes them. The host supplies all totals at the same boundary.
pub(crate) fn settle_sources<I: Identifiable, N: FixedNumber>(
    protocol: &Protocol,
    node: &mut Node<I, N>,
    lanes: &[(Lane<N>, N)],
    positions: &mut [(LaneId, LaneNode<N>)],
) -> Result<()> {
    if node.depth > protocol.terms.max_depth() {
        return Err(Error::InvalidDepth);
    }
    if lanes.len() != positions.len() || positions.len() > usize::from(crate::MAX_LANES) {
        return Err(Error::IncompleteSources);
    }
    let mut supplied = LaneMask::default();
    for (id, _) in positions.iter() {
        protocol.check_lane(*id)?;
        if supplied.contains(*id) || lanes.iter().filter(|(lane, _)| lane.id == *id).count() != 1 {
            return Err(Error::IncompleteSources);
        }
        supplied.insert(*id);
    }
    if supplied != node.lanes {
        return Err(Error::IncompleteSources);
    }
    for (id, position) in positions {
        let (lane, total_earned) = lanes
            .iter()
            .find(|(lane, _)| lane.id == *id)
            .ok_or(Error::IncompleteSources)?;
        let index = lane.reward_index(*total_earned)?;
        let (own, overrides) = position.settle(protocol.terms, node.depth, index)?;
        node.earn(own, overrides)?;
    }
    Ok(())
}

/// Commit supplied lane earnings before closing shared residuals. Clearing only
/// the parent's family carry would let old lane dust arrive under new terms.
pub(crate) fn close_sources<N: FixedNumber>(lanes: &mut [(Lane<N>, N)]) -> Result<()> {
    for (lane, total_earned) in lanes {
        let (mut next, _) = lane.clone().settle(*total_earned)?;
        next.pool.close_eligibility()?;
        *lane = next;
    }
    Ok(())
}

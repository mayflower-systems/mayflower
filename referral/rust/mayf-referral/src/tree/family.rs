use mayf_fixed_number::FixedNumber;

use crate::{
    Emission, Error, Identifiable, Lane, LaneId, LaneMask, LaneNode, Node, Protocol, Result,
};

/// A complete source barrier: settle every lane ever materialized for this
/// parent at one emission snapshot. This helper projects global lanes without
/// writing them; an eligibility change subsequently commits and closes them.
pub(crate) fn settle_sources<I: Identifiable, N: FixedNumber>(
    protocol: &Protocol<N>,
    node: &mut Node<I, N>,
    lanes: &[Lane<N>],
    positions: &mut [(LaneId, LaneNode<N>)],
    emission: Emission<N>,
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
        if supplied.contains(*id) || lanes.iter().filter(|lane| lane.id == *id).count() != 1 {
            return Err(Error::IncompleteSources);
        }
        supplied.insert(*id);
    }
    if supplied != node.lanes {
        return Err(Error::IncompleteSources);
    }
    let gauge = protocol.gauge_at(emission)?;
    for (id, position) in positions {
        let lane = lanes
            .iter()
            .find(|lane| lane.id == *id)
            .ok_or(Error::IncompleteSources)?;
        let index = lane.reward_index(&gauge)?;
        let (own, overrides) = position.settle(protocol.terms, node.depth, index)?;
        node.earn(own, overrides)?;
    }
    Ok(())
}

/// Commit the old upstream intervals before closing shared residuals. Clearing
/// only the parent's family carry would let old gauge/lane dust arrive later
/// under new family terms. This deliberately writes bounded shared records.
pub(crate) fn close_sources<N: FixedNumber>(
    protocol: &mut Protocol<N>,
    lanes: &mut [Lane<N>],
    emission: Emission<N>,
) -> Result<()> {
    let mut gauge = protocol.gauge_at(emission)?;
    for lane in lanes {
        let (mut next, _) = lane.clone().settle(&gauge)?;
        next.pool.close_eligibility()?;
        *lane = next;
    }
    gauge.close_eligibility()?;
    protocol.gauge = gauge;
    Ok(())
}

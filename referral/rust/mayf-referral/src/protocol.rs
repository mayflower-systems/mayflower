use crate::{Error, LaneId, NetworkTerms, Result};

/// Immutable tree terms and the registry of permanent asset lanes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Protocol {
    pub terms: NetworkTerms,
    /// Registered lane IDs are the consecutive slots `0..lane_count`.
    pub lane_count: u8,
}

impl Protocol {
    pub(crate) fn check_lane(&self, lane: LaneId) -> Result<()> {
        if lane.get() >= self.lane_count {
            return Err(Error::InvalidLane);
        }
        Ok(())
    }
}

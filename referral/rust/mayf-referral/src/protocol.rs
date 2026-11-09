use mayf_fixed_number::FixedNumber;

use crate::{Emission, Error, Gauge, LaneId, NetworkTerms, Result};

/// Immutable tree terms and the allocation of emission among permanent lanes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Protocol<N> {
    pub terms: NetworkTerms,
    /// Registered lane IDs are the consecutive slots `0..lane_count`.
    pub lane_count: u8,
    pub gauge: Gauge<N>,
}

impl<N: FixedNumber> Protocol<N> {
    pub(crate) fn gauge_at(&self, emission: Emission<N>) -> Result<Gauge<N>> {
        self.gauge.settled(emission)
    }

    pub(crate) fn check_lane(&self, lane: LaneId) -> Result<()> {
        if lane.get() >= self.lane_count {
            return Err(Error::InvalidLane);
        }
        Ok(())
    }
}

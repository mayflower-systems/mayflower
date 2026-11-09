use mayf_fixed_number::FixedNumber;

use crate::tree::LanePool;
use crate::weighting::{Gauge, LaneGauge};
use crate::{Error, LaneId, Result, Seconds};

/// One lane: one deposit token, such as Mayflower-ETH. The host stores one per lane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lane<N> {
    pub id: LaneId,
    pub gauge: LaneGauge<N>,
    pub pool: LanePool<N>,
    pub settled_at: Seconds,
}

impl<N: FixedNumber> Lane<N> {
    /// Take the lane's allocation from `gauge`, which is settled, and pay it across the lane's
    /// balance. Returns the lane, and its reward index.
    pub(crate) fn settle(self, gauge: &Gauge<N>) -> Result<(Self, N)> {
        if gauge.emission_seen.at < self.settled_at {
            return Err(Error::TimeWentBack);
        }
        let (lane_gauge, allocation) = self.gauge.take_allocation(gauge)?;
        let pool = self.pool.absorb(allocation)?;
        let reward_index = pool.reward_index;

        let lane = Self {
            id: self.id,
            gauge: lane_gauge,
            pool,
            settled_at: gauge.emission_seen.at,
        };
        Ok((lane, reward_index))
    }

    /// The lane's reward index at `gauge`, which is settled, without changing the lane
    pub(crate) fn reward_index(&self, gauge: &Gauge<N>) -> Result<N> {
        let (_, reward_index) = self.clone().settle(gauge)?;
        Ok(reward_index)
    }
}

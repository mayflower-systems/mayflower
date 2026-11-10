use mayf_fixed_number::FixedNumber;

use crate::tree::LanePool;
use crate::{Error, LaneId, Result};

/// One asset's eligible balances and rewards. The host supplies cumulative
/// earnings for this lane independently of how those earnings are produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lane<N> {
    pub id: LaneId,
    pub pool: LanePool<N>,
    /// Cumulative earnings already distributed or recorded as unallocated.
    /// Registration starts this cursor at the host's supplied baseline.
    pub total_earned_seen: N,
}

impl<N: FixedNumber> Lane<N> {
    /// Distribute only the growth in cumulative earnings at the old balance.
    /// An empty lane still consumes its earnings, preventing later backpay.
    pub(crate) fn settle(self, total_earned: N) -> Result<(Self, N)> {
        if self.total_earned_seen < N::ZERO || total_earned < self.total_earned_seen {
            return Err(Error::InvalidEarnings);
        }
        let allocation = total_earned.checked_sub(self.total_earned_seen)?;
        let pool = self.pool.absorb(allocation)?;
        let reward_index = pool.reward_index;

        let lane = Self {
            id: self.id,
            pool,
            total_earned_seen: total_earned,
        };
        Ok((lane, reward_index))
    }

    /// Project the reward index at the supplied total without changing the lane.
    pub(crate) fn reward_index(&self, total_earned: N) -> Result<N> {
        let (_, reward_index) = self.clone().settle(total_earned)?;
        Ok(reward_index)
    }
}

#[cfg(test)]
mod tests;

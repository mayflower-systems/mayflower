use mayf_fixed_number::FixedNumber;

use super::Gauge;
use crate::numbers::Change;
use crate::{Amount, Error, Result};

/// One lane's place in the gauge: its weight, and the gauge's weight index when it last took its
/// allocation
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneGauge<N> {
    /// The sum of the lane's gauge votes
    pub weight: Amount,
    pub weight_index_seen: N,
}

/// A new lane, with no weight. Its seen index does not matter yet: a lane with no weight earns
/// nothing, and the vote that gives it weight first takes its allocation, which updates the index.
impl<N: FixedNumber> Default for LaneGauge<N> {
    fn default() -> Self {
        Self {
            weight: 0,
            weight_index_seen: N::ZERO,
        }
    }
}

impl<N: FixedNumber> LaneGauge<N> {
    /// Take the credits allocated to this lane since it last took them, from `gauge`, which is
    /// settled. Returns the lane, and its allocation.
    pub(crate) fn take_allocation(mut self, gauge: &Gauge<N>) -> Result<(Self, N)> {
        if gauge.weight_index < self.weight_index_seen {
            return Err(Error::IndexWentBack);
        }
        let growth = gauge.weight_index.checked_sub(self.weight_index_seen)?;
        // An integer lane weight introduces no additional rounding.
        let allocation = growth.checked_mul_floor(N::checked_from_u64(self.weight)?)?;

        self.weight_index_seen = gauge.weight_index;
        Ok((self, allocation))
    }

    pub(super) fn change_weight(mut self, change: Change) -> Result<Self> {
        self.weight = change.apply(self.weight)?;
        Ok(self)
    }
}

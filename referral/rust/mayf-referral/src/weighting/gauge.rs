use core::num::NonZero;

use mayf_fixed_number::DirectedRounding::Floor;
use mayf_fixed_number::{FixedNumber, MathResult};

use super::LaneGauge;
use crate::Amount;
use crate::numbers::Change;

/// The split of all emission among lanes, in proportion to their gauge votes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gauge<N> {
    /// The total emission the gauge has already split, or skipped while no lane had votes
    pub emission_seen: Amount,
    /// Credits per gauge vote since the network began
    pub weight_index: N,
    /// The sum of every lane's weight
    pub total_weight: Amount,
}

impl<N: FixedNumber> Default for Gauge<N> {
    fn default() -> Self {
        Self {
            emission_seen: 0,
            weight_index: N::ZERO,
            total_weight: 0,
        }
    }
}

impl<N: FixedNumber> Gauge<N> {
    /// The gauge after splitting all emission up to `emitted`.
    /// While no lane has votes, nothing is emitted: the gauge skips that emission.
    pub(crate) fn settled(&self, emitted: Amount) -> MathResult<Self> {
        let fresh = emitted.saturating_sub(self.emission_seen);
        let mut gauge = self.clone();

        if let Some(total_weight) = NonZero::new(self.total_weight) {
            let per_vote = N::checked_from_ratio_round(fresh.into(), total_weight.into(), Floor)?;
            gauge.weight_index = gauge.weight_index.checked_add(per_vote)?;
        }

        gauge.emission_seen = emitted.max(self.emission_seen);
        Ok(gauge)
    }

    /// Move one voter's votes on `lane` from `old` to `new`, after `emitted`.
    /// Returns the next gauge and lane, and the credits the lane earned at its old weight.
    pub(crate) fn revote(
        self,
        lane: LaneGauge<N>,
        old: Amount,
        new: Amount,
        emitted: Amount,
    ) -> MathResult<(Self, LaneGauge<N>, N)> {
        let change = Change::between(old, new);
        let mut gauge = self.settled(emitted)?;
        let (lane, allocation) = lane.take_allocation(&gauge)?;
        gauge.total_weight = change.apply(gauge.total_weight);
        Ok((gauge, lane.change_weight(change), allocation))
    }
}

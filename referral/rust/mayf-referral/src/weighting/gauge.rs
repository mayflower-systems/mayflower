use mayf_fixed_number::FixedNumber;

use super::LaneGauge;
use crate::numbers::Change;
use crate::{Amount, Emission, Result};

/// The split of all emission among lanes, in proportion to their gauge votes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gauge<N> {
    /// The cumulative schedule already routed or recorded as unallocated.
    pub emission_seen: Emission<N>,
    /// Credits per gauge vote since the network began
    pub weight_index: N,
    /// The sum of every lane's weight
    pub total_weight: Amount,
    /// Emission below one index quantum at the current voting weights.
    pub remainder: N,
    pub unallocated: N,
}

impl<N: FixedNumber> Default for Gauge<N> {
    fn default() -> Self {
        Self {
            emission_seen: Emission::default(),
            weight_index: N::ZERO,
            total_weight: 0,
            remainder: N::ZERO,
            unallocated: N::ZERO,
        }
    }
}

impl<N: FixedNumber> Gauge<N> {
    /// Route new scheduled emission under the existing weights.
    pub(crate) fn settled(&self, emission: Emission<N>) -> Result<Self> {
        emission.validate_after(self.emission_seen)?;
        let fresh = emission.total.checked_sub(self.emission_seen.total)?;
        let mut gauge = self.clone();
        let available = gauge.remainder.checked_add(fresh)?;

        if gauge.total_weight == 0 {
            gauge.unallocated = gauge.unallocated.checked_add(available)?;
            gauge.remainder = N::ZERO;
        } else {
            let weight = N::checked_from_u64(gauge.total_weight)?;
            let increment = available.checked_div_floor(weight)?;
            // Multiplication by the integer weight is exact on the fixed-point grid.
            let routed = increment.checked_mul_floor(weight)?;
            gauge.weight_index = gauge.weight_index.checked_add(increment)?;
            gauge.remainder = available.checked_sub(routed)?;
        }

        gauge.emission_seen = emission;
        Ok(gauge)
    }

    /// End an eligibility interval after settling its emission. Even a change
    /// farther down the allocation chain must not inherit this shared residue.
    pub(crate) fn close_eligibility(&mut self) -> Result<()> {
        self.unallocated = self.unallocated.checked_add(self.remainder)?;
        self.remainder = N::ZERO;
        Ok(())
    }

    /// Move one voter's votes after routing the old interval.
    /// Returns the next gauge and lane, and the credits the lane earned at its old weight.
    pub(crate) fn revote(
        self,
        lane: LaneGauge<N>,
        old: Amount,
        new: Amount,
        emission: Emission<N>,
    ) -> Result<(Self, LaneGauge<N>, N)> {
        Change::Remove(old).apply(self.total_weight)?;
        Change::Remove(old).apply(lane.weight)?;
        let change = Change::between(old, new);
        let mut gauge = self.settled(emission)?;
        let (lane, allocation) = lane.take_allocation(&gauge)?;
        if old != new {
            gauge.close_eligibility()?;
        }
        gauge.total_weight = change.apply(gauge.total_weight)?;
        Ok((gauge, lane.change_weight(change)?, allocation))
    }
}

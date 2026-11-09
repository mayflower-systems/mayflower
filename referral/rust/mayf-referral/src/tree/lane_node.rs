use mayf_fixed_number::FixedNumber;

use super::terms::apply_rate;
use super::{Depth, NetworkTerms};
use crate::numbers::Change;
use crate::{Amount, Error, Result};

/// Exact gross overrides in credits times the micro-basis-point denominator.
/// This representation delays rounding until the family's old sharing policy
/// has attributed every fraction to either its parent or its marketing pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Override<N>(pub(super) N);

/// A member's asset balance and strict-descendant total in one lane.
///
/// The host binds this record to its `(lane, member)` storage key. Once a
/// position is materialized it remains in the member's historical lane mask,
/// even after every balance is withdrawn. Family accounting lives on `Node`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneNode<N> {
    pub balance: Amount,
    pub downline: Amount,
    pub reward_index_seen: N,
    /// Individual own-yield remainder, in micro-basis-point numerator units.
    /// It remains this member's property across balance and family changes.
    pub own_carry: N,
}

impl<N: FixedNumber> Default for LaneNode<N> {
    fn default() -> Self {
        Self::at(N::ZERO)
    }
}

impl<N: FixedNumber> LaneNode<N> {
    /// Materialize a position at the current index, without historical yield.
    pub(crate) fn at(reward_index: N) -> Self {
        Self {
            balance: 0,
            downline: 0,
            reward_index_seen: reward_index,
            own_carry: N::ZERO,
        }
    }

    /// Settle the old asset balances, returning own yield and gross overrides
    /// separately. The caller passes both to `Node::earn` under the current
    /// family terms before changing any balances or family eligibility.
    pub(crate) fn settle(
        &mut self,
        terms: NetworkTerms,
        depth: Depth,
        reward_index: N,
    ) -> Result<(N, Override<N>)> {
        if reward_index < self.reward_index_seen {
            return Err(Error::IndexWentBack);
        }
        let own_rate = terms.own_rate(depth)?;
        let growth = reward_index.checked_sub(self.reward_index_seen)?;
        let own_gross = growth.checked_mul_floor(N::checked_from_u64(self.balance)?)?;
        let downline_gross = growth.checked_mul_floor(N::checked_from_u64(self.downline)?)?;
        let own = apply_rate(own_gross, own_rate, &mut self.own_carry)?;
        let overrides = downline_gross
            .checked_mul_floor(N::checked_from_u64(u64::from(terms.override_rate().get()))?)?;
        self.reward_index_seen = reward_index;
        Ok((own, Override(overrides)))
    }

    pub(crate) fn change_balance(&mut self, change: Change) -> crate::Result<()> {
        self.balance = change.apply(self.balance)?;
        Ok(())
    }

    pub(crate) fn change_downline(&mut self, change: Change) -> crate::Result<()> {
        self.downline = change.apply(self.downline)?;
        Ok(())
    }
}

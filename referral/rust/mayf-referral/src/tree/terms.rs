use mayf_fixed_number::DirectedRounding::Ceil;
use mayf_fixed_number::{FixedNumber, MathResult};
use mayf_micro_basis_points::{self as micro_basis_points, MicroBasisPoints};

use super::Depth;
use crate::{Amount, Error, Result};

/// The network's fixed terms: the deepest depth a member can have, and the override each
/// member pays on its points to each member of its upline
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkTerms {
    max_depth: Depth,
    override_rate: MicroBasisPoints,
}

impl NetworkTerms {
    /// A member at depth d keeps 1 - d × override_rate of its own points, so that must stay
    /// positive at the maximum depth.
    pub fn new(max_depth: Depth, override_rate: MicroBasisPoints) -> Result<Self> {
        if u32::from(max_depth) * override_rate.get() >= micro_basis_points::ONE {
            return Err(Error::OverrideRateTooLargeForDepth);
        }

        Ok(Self {
            max_depth,
            override_rate,
        })
    }

    pub fn max_depth(&self) -> Depth {
        self.max_depth
    }

    pub fn override_rate(&self) -> MicroBasisPoints {
        self.override_rate
    }

    /// A member's core points at `depth`: its balance, minus one override for each upline
    /// member, plus one override on each unit of its downline's balance.
    ///
    /// The rate rounds up, so upline members never get less than the stated rate. Each
    /// override is the rate times a whole balance, which is exact, so the core points of all
    /// members sum to the lane's balance.
    pub(super) fn core_points<N: FixedNumber>(
        &self,
        depth: usize,
        balance: Amount,
        downline: Amount,
    ) -> MathResult<N> {
        let rate: N = self.override_rate.to_fixed(Ceil)?;
        let upline_rate = rate.checked_mul_floor(N::checked_from_u64(depth as Amount)?)?;
        let balance = N::checked_from_u64(balance)?;
        let paid_upline = upline_rate.checked_mul_floor(balance)?;
        let received = rate.checked_mul_floor(N::checked_from_u64(downline)?)?;

        balance.checked_sub(paid_upline)?.checked_add(received)
    }
}

use mayf_fixed_number::{FixedNumber, IntegralRounding, MathError, MathResult};
use mayf_micro_basis_points::{self as micro_basis_points, MicroBasisPoints};

use super::lane_node::Override;
use super::{Depth, Identifiable, Offer};
use crate::{Amount, Error, LaneMask, Result};

/// Permanent membership, collected-across-lanes income, and one family's
/// credit-escrow marketing pool. Received gifts are terminal income.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node<I, N> {
    pub id: I,
    /// `None` identifies an earning top-level seat under the protocol root.
    pub parent_id: Option<I>,
    pub depth: Depth,
    /// Every lane ever materialized for this member, including withdrawn ones.
    pub lanes: LaneMask,
    pub offer: Offer,
    /// The prospective share of gross overrides offered to direct children.
    pub sharing: MicroBasisPoints,
    /// Individual earned credits not yet collected, including terminal gifts.
    pub kept: N,
    /// Issued credits this member escrows under its immediate parent.
    pub escrow: Amount,
    /// Sum of the issued credits escrowed by this member's direct children.
    pub child_escrow: Amount,
    /// Shared credits per unit of direct-child credit escrow, across all lanes.
    pub gift_index: N,
    pub parent_gift_index_seen: N,
    /// Individual retained-override remainder, in `1_000_000²` numerator units.
    pub retained_carry: N,
    /// Shared marketing remainder, in `1_000_000²` numerator units, before division
    /// among child escrow. Closed whenever sharing or eligibility changes.
    pub marketing_carry: N,
    /// Funded marketing credits too small to advance the family index yet.
    pub gift_remainder: N,
    /// Closed, representable family-index dust. It is not anyone's earnings.
    pub unallocated: N,
    /// Closed marketing-rate dust in exact numerator units. Its credit value
    /// is this cumulative counter divided by `1_000_000²`.
    pub unallocated_numerator: N,
}

impl<I: Identifiable, N: FixedNumber> Node<I, N> {
    pub(crate) fn top_level(id: I, offer: Offer) -> Self {
        Self::joined(id, None, 0, offer, N::ZERO)
    }

    /// Validate the complete parent-first lineage, including cached depths
    /// and uniqueness. A top-level member has an empty lineage.
    pub(crate) fn validate_lineage(&self, max_depth: Depth, lineage: &[Self]) -> Result<()> {
        if self.depth > max_depth {
            return Err(Error::InvalidDepth);
        }
        if lineage.len() != usize::from(self.depth) {
            return Err(Error::BrokenLineage);
        }
        let mut parent_id = self.parent_id;
        for (index, node) in lineage.iter().enumerate() {
            let expected_depth = usize::from(self.depth) - index - 1;
            if parent_id != Some(node.id)
                || usize::from(node.depth) != expected_depth
                || node.id == self.id
                || lineage[..index].iter().any(|prior| prior.id == node.id)
            {
                return Err(Error::BrokenLineage);
            }
            parent_id = node.parent_id;
        }
        if parent_id.is_some() {
            return Err(Error::BrokenLineage);
        }
        Ok(())
    }

    /// Admit a zero-escrow child at the posted fee. The host checks unique
    /// membership, and the action additionally checks the network depth cap.
    pub(crate) fn admit(self, id: I, offer: Offer) -> Result<(Self, Self, Amount)> {
        let depth = self.depth.checked_add(1).ok_or(Error::ParentAtMaxDepth)?;
        if id == self.id {
            return Err(Error::BrokenLineage);
        }
        let child = Self::joined(id, Some(self.id), depth, offer, self.gift_index);
        let (raised, fee) = self.offer.charge();
        let parent = Self {
            offer: raised,
            ..self
        };
        Ok((parent, child, fee))
    }

    /// Credit terminal gifts using the old escrow balance. The checkpoint
    /// advances even with zero escrow, so a later deposit cannot claim history.
    pub(crate) fn receive_gift(&mut self, parent_index: N) -> Result<()> {
        if parent_index < self.parent_gift_index_seen {
            return Err(Error::IndexWentBack);
        }
        let growth = parent_index.checked_sub(self.parent_gift_index_seen)?;
        let gifts = growth.checked_mul_floor(N::checked_from_u64(self.escrow)?)?;
        self.kept = self.kept.checked_add(gifts)?;
        self.parent_gift_index_seen = parent_index;
        Ok(())
    }

    /// Credit own yield directly; split only gross overrides into retained
    /// income and the current family's marketing budget. Independent carries
    /// keep old fractions funded instead of first crediting them to the parent
    /// and later attempting to subtract them when a marketing quantum accrues.
    pub(crate) fn earn(&mut self, own: N, overrides: Override<N>) -> MathResult<()> {
        if own < N::ZERO || overrides.0 < N::ZERO {
            return Err(MathError::Domain);
        }
        let sharing = if self.child_escrow == 0 {
            0
        } else {
            self.sharing.get()
        };
        let retained = Self::allocate_override(
            overrides.0,
            micro_basis_points::ONE - sharing,
            &mut self.retained_carry,
        )?;
        let marketed = Self::allocate_override(overrides.0, sharing, &mut self.marketing_carry)?;
        self.kept = self.kept.checked_add(own)?.checked_add(retained)?;

        if self.child_escrow != 0 {
            let escrow = N::checked_from_u64(self.child_escrow)?;
            let available = self.gift_remainder.checked_add(marketed)?;
            let increment = available.checked_div_floor(escrow)?;
            self.gift_remainder = available.checked_sub(increment.checked_mul_floor(escrow)?)?;
            self.gift_index = self.gift_index.checked_add(increment)?;
        }
        Ok(())
    }

    /// Both integer rates are applied before rounding. The fraction assigned
    /// to the parent therefore stays individually owned through family changes.
    fn allocate_override(gross_numerator: N, rate: u32, carry: &mut N) -> MathResult<N> {
        let numerator = gross_numerator
            .checked_mul_floor(N::checked_from_u64(u64::from(rate))?)?
            .checked_add(*carry)?;
        let denominator = N::checked_from_u64(u64::from(micro_basis_points::ONE).pow(2))?;
        let amount = numerator.checked_div_floor(denominator)?;
        *carry = numerator.checked_sub(amount.checked_mul_floor(denominator)?)?;
        Ok(amount)
    }

    /// Close shared dust after the complete old-terms source barrier, before
    /// changing sharing or any child's escrow. Individual carry remains owned
    /// by this member. Neither closed remainder becomes later family income.
    pub(crate) fn close_family(&mut self) -> MathResult<()> {
        self.unallocated = self.unallocated.checked_add(self.gift_remainder)?;
        self.unallocated_numerator = self
            .unallocated_numerator
            .checked_add(self.marketing_carry)?;
        self.gift_remainder = N::ZERO;
        self.marketing_carry = N::ZERO;
        Ok(())
    }

    /// Collect whole credit base units and preserve the individual's fraction.
    pub(crate) fn collect(mut self) -> MathResult<(Self, Amount)>
    where
        N: IntegralRounding,
    {
        let whole = self.kept.checked_floor_to_u64()?;
        self.kept = self.kept.checked_sub(N::checked_from_u64(whole)?)?;
        Ok((self, whole))
    }

    fn joined(
        id: I,
        parent_id: Option<I>,
        depth: Depth,
        offer: Offer,
        parent_gift_index: N,
    ) -> Self {
        Self {
            id,
            parent_id,
            depth,
            lanes: LaneMask::default(),
            offer,
            sharing: MicroBasisPoints::ZERO,
            kept: N::ZERO,
            escrow: 0,
            child_escrow: 0,
            gift_index: N::ZERO,
            parent_gift_index_seen: parent_gift_index,
            retained_carry: N::ZERO,
            marketing_carry: N::ZERO,
            gift_remainder: N::ZERO,
            unallocated: N::ZERO,
            unallocated_numerator: N::ZERO,
        }
    }
}

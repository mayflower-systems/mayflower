//! Immutable floor/ramp/main geometry and its legal transformations.
//!
//! Geometry is expressed in unshifted curve coordinates. It knows nothing
//! about characteristics, calculators, live supply, reserve policy, or supply
//! issued at the floor.

use mayf_fixed_number::{AboveOne, FixedNumber, NonNeg, Pos};

/// A structural failure while constructing or transforming segmentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentationError {
    /// The ramp start must not lie after the ramp end.
    RampStartsAfterEnd,
    /// A stored or proposed ramp width extends left of curve-coordinate zero.
    RampWidthExceedsEnd,
    /// A floor-raise transformation cannot lower the floor.
    FloorDecrease,
    /// Checked coordinate arithmetic could not produce a representable value.
    ArithmeticFailure,
}

/// Segmented geometry proposed for a floor raise.
///
/// This is a constructive representation of every structurally legal reshape:
///
/// - the new ramp start is the old start plus `ramp_start_advance_by`, which is
///   strictly positive;
/// - the new ramp end is `max(old end, new start)` plus
///   `ramp_end_extension_by`, which is non-negative.
///
/// Consequently a proposal cannot encode a retreating boundary, a ramp whose
/// start lies after its end, or a floor raise whose ramp start does not
/// advance. A zero extension can keep the old ramp end fixed or collapse the
/// ramp at an advanced start. Economic acceptance of the resulting candidate
/// belongs to the calling action. For any legal target `(start, end)`, the
/// inverse construction is `start - old_start` and
/// `end - max(old_end, start)`, so the representation loses no valid geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentedRaiseFloorProposal<N>
where
    N: FixedNumber,
{
    ramp_start_advance_by: Pos<N>,
    ramp_end_extension_by: NonNeg<N>,
}

impl<N> SegmentedRaiseFloorProposal<N>
where
    N: FixedNumber,
{
    pub const fn new(ramp_start_advance_by: Pos<N>, ramp_end_extension_by: NonNeg<N>) -> Self {
        Self {
            ramp_start_advance_by,
            ramp_end_extension_by,
        }
    }

    pub const fn ramp_start_advance_by(&self) -> Pos<N> {
        self.ramp_start_advance_by
    }

    /// Extension beyond both the old ramp end and the proposed ramp start.
    pub const fn ramp_end_extension_by(&self) -> NonNeg<N> {
        self.ramp_end_extension_by
    }
}

/// Canonical floor/ramp/main boundaries in unshifted curve coordinates.
///
/// The value is immutable and valid by construction: both boundaries are
/// non-negative, `ramp_start <= ramp_end`, and the ramp scalar is above one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "segmentation transformations return a new immutable value"]
pub struct Segmentation<N>
where
    N: FixedNumber,
{
    floor: NonNeg<N>,
    ramp_start: NonNeg<N>,
    ramp_end: NonNeg<N>,
    ramp_scalar: AboveOne<N>,
}

impl<N> Segmentation<N>
where
    N: FixedNumber,
{
    /// Construct geometry from its two boundaries.
    pub fn new(
        floor: NonNeg<N>,
        ramp_start: NonNeg<N>,
        ramp_end: NonNeg<N>,
        ramp_scalar: AboveOne<N>,
    ) -> Result<Self, SegmentationError> {
        if ramp_start > ramp_end {
            return Err(SegmentationError::RampStartsAfterEnd);
        }

        Ok(Self {
            floor,
            ramp_start,
            ramp_end,
            ramp_scalar,
        })
    }

    /// Construct geometry from the storage-oriented ramp end and width.
    pub fn from_end_width(
        floor: NonNeg<N>,
        ramp_end: NonNeg<N>,
        ramp_width: NonNeg<N>,
        ramp_scalar: AboveOne<N>,
    ) -> Result<Self, SegmentationError> {
        if ramp_width > ramp_end {
            return Err(SegmentationError::RampWidthExceedsEnd);
        }
        let ramp_start = ramp_end
            .checked_sub(ramp_width)
            .map_err(|_| SegmentationError::ArithmeticFailure)?;

        Self::new(floor, ramp_start, ramp_end, ramp_scalar)
    }

    pub const fn floor(&self) -> NonNeg<N> {
        self.floor
    }

    pub const fn ramp_start(&self) -> NonNeg<N> {
        self.ramp_start
    }

    pub const fn ramp_end(&self) -> NonNeg<N> {
        self.ramp_end
    }

    pub const fn ramp_scalar(&self) -> AboveOne<N> {
        self.ramp_scalar
    }

    pub fn ramp_width(&self) -> Result<NonNeg<N>, SegmentationError> {
        self.ramp_end
            .checked_sub(self.ramp_start)
            .map_err(|_| SegmentationError::ArithmeticFailure)
    }

    /// Return floor-raised geometry without mutating this value.
    pub fn raise_floor(
        self,
        new_floor: NonNeg<N>,
        proposal: SegmentedRaiseFloorProposal<N>,
    ) -> Result<Self, SegmentationError> {
        raise_floor(self, new_floor, proposal)
    }

    /// Boundaries the proposal would produce, without constructing geometry.
    ///
    /// Callers that must evaluate the proposed coordinates before the new
    /// floor is known share this validation with [`Self::raise_floor`].
    pub fn proposed_boundaries(
        &self,
        proposal: SegmentedRaiseFloorProposal<N>,
    ) -> Result<(NonNeg<N>, NonNeg<N>), SegmentationError> {
        let new_ramp_start = self
            .ramp_start
            .checked_add(NonNeg::from(proposal.ramp_start_advance_by()))
            .map_err(|_| SegmentationError::ArithmeticFailure)?;
        let ramp_end_base = self.ramp_end.max(new_ramp_start);
        let new_ramp_end = ramp_end_base
            .checked_add(proposal.ramp_end_extension_by())
            .map_err(|_| SegmentationError::ArithmeticFailure)?;

        Ok((new_ramp_start, new_ramp_end))
    }

    /// Return geometry contracted to an unshifted curve-supply coordinate.
    pub fn contract_to_curve_supply(self, curve_supply: NonNeg<N>) -> Self {
        contract_to_curve_supply(self, curve_supply)
    }
}

/// Return floor-raised geometry without mutating `current`.
pub fn raise_floor<N>(
    current: Segmentation<N>,
    new_floor: NonNeg<N>,
    proposal: SegmentedRaiseFloorProposal<N>,
) -> Result<Segmentation<N>, SegmentationError>
where
    N: FixedNumber,
{
    if new_floor == current.floor {
        return Ok(current);
    }
    if new_floor < current.floor {
        return Err(SegmentationError::FloorDecrease);
    }

    let (new_ramp_start, new_ramp_end) = current.proposed_boundaries(proposal)?;

    Segmentation::new(new_floor, new_ramp_start, new_ramp_end, current.ramp_scalar)
}

/// Return geometry contracted to `curve_supply` without mutating `current`.
pub fn contract_to_curve_supply<N>(
    current: Segmentation<N>,
    curve_supply: NonNeg<N>,
) -> Segmentation<N>
where
    N: FixedNumber,
{
    if curve_supply >= current.ramp_end {
        return current;
    }

    let ramp_start = if curve_supply <= current.ramp_start {
        curve_supply
    } else {
        current.ramp_start
    };

    Segmentation {
        floor: current.floor,
        ramp_start,
        ramp_end: curve_supply,
        ramp_scalar: current.ramp_scalar,
    }
}

//! Characteristic-free tests for segmented geometry and its transformations.

use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{AboveOne, FixedNumber, NonNeg, Pos};
use mayf_schedule_segmented::geometry::{contract_to_curve_supply, raise_floor};
use mayf_schedule_segmented::{Segmentation, SegmentationError, SegmentedRaiseFloorProposal};

type Fx = I64F64;

fn n(value: i64) -> Fx {
    Fx::checked_from_i128(i128::from(value)).unwrap()
}

fn nn(value: i64) -> NonNeg<Fx> {
    NonNeg::new(n(value)).unwrap()
}

fn pos(value: i64) -> Pos<Fx> {
    Pos::new(n(value)).unwrap()
}

fn proposal(
    ramp_start_advance_by: i64,
    ramp_end_extension_by: i64,
) -> SegmentedRaiseFloorProposal<Fx> {
    SegmentedRaiseFloorProposal::new(pos(ramp_start_advance_by), nn(ramp_end_extension_by))
}

fn scalar() -> AboveOne<Fx> {
    AboveOne::new(n(2)).unwrap()
}

fn segmentation() -> Segmentation<Fx> {
    Segmentation::new(nn(5), nn(2), nn(5), scalar()).unwrap()
}

#[test]
fn construction_establishes_boundary_order() {
    let from_boundaries = segmentation();
    let from_storage = Segmentation::from_end_width(nn(5), nn(5), nn(3), scalar()).unwrap();

    assert_eq!(from_storage, from_boundaries);
    assert_eq!(from_storage.ramp_width().unwrap(), nn(3));
    assert_eq!(
        Segmentation::new(nn(5), nn(6), nn(5), scalar()),
        Err(SegmentationError::RampStartsAfterEnd)
    );
    assert_eq!(
        Segmentation::from_end_width(nn(5), nn(3), nn(4), scalar()),
        Err(SegmentationError::RampWidthExceedsEnd)
    );
}

#[test]
fn floor_raise_is_a_pure_geometry_transformation() {
    let current = segmentation();
    let proposal = proposal(4, 3);

    let from_function = raise_floor(current, nn(7), proposal).unwrap();
    let from_method = current.raise_floor(nn(7), proposal).unwrap();

    assert_eq!(from_function, from_method);
    assert_eq!(from_function.floor(), nn(7));
    assert_eq!(from_function.ramp_start(), nn(6));
    assert_eq!(from_function.ramp_end(), nn(9));
    assert_eq!(from_function.ramp_scalar(), current.ramp_scalar());
    assert_eq!(current, segmentation());
}

#[test]
fn floor_raise_accepts_collapsed_and_advanced_ramps() {
    let current = segmentation();
    let raised = current.raise_floor(nn(7), proposal(5, 0)).unwrap();

    assert_eq!(raised.ramp_start(), nn(7));
    assert_eq!(raised.ramp_end(), nn(7));
    assert_eq!(raised.ramp_width().unwrap(), nn(0));
    assert!(raised.ramp_end() > current.ramp_end());
}

#[test]
fn floor_raise_proposals_are_valid_by_construction() {
    let current = segmentation();

    assert_eq!(
        current.raise_floor(nn(4), proposal(1, 0)),
        Err(SegmentationError::FloorDecrease)
    );

    // Fixed end, collapse at the old end, and collapse beyond the old end are
    // all legal points in the constructive coordinate space.
    let cases = [
        (proposal(1, 0), nn(3), nn(5)),
        (proposal(3, 0), nn(5), nn(5)),
        (proposal(5, 0), nn(7), nn(7)),
        (proposal(4, 3), nn(6), nn(9)),
    ];
    for (proposal, expected_start, expected_end) in cases {
        let (new_start, new_end) = current.proposed_boundaries(proposal).unwrap();
        assert_eq!((new_start, new_end), (expected_start, expected_end));
        assert!(new_start > current.ramp_start());
        assert!(new_end >= current.ramp_end());
        assert!(new_start <= new_end);
    }

    let ignored_proposal = proposal(1, 0);
    assert_eq!(
        current.raise_floor(current.floor(), ignored_proposal),
        Ok(current)
    );
}

#[test]
fn contraction_moves_only_boundaries_to_the_right_of_curve_supply() {
    let current = segmentation();
    let cases = [
        (nn(7), nn(2), nn(5)),
        (nn(4), nn(2), nn(4)),
        (nn(2), nn(2), nn(2)),
        (nn(1), nn(1), nn(1)),
        (nn(0), nn(0), nn(0)),
    ];

    for (curve_supply, expected_start, expected_end) in cases {
        let contracted = contract_to_curve_supply(current, curve_supply);
        assert_eq!(contracted, current.contract_to_curve_supply(curve_supply));
        assert_eq!(contracted.ramp_start(), expected_start);
        assert_eq!(contracted.ramp_end(), expected_end);
        assert_eq!(contracted.floor(), current.floor());
        assert_eq!(contracted.ramp_scalar(), current.ramp_scalar());
        assert!(contracted.ramp_start() <= contracted.ramp_end());
        assert!(contracted.ramp_end() <= current.ramp_end());
    }
}

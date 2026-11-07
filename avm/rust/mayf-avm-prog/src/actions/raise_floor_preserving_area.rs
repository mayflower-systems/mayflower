use mayf_avm_calculator::{CharacteristicEngine, FloorRaisePolicy, FloorRaiseProposal};
use mayf_fixed_number::{NonNeg, Pos, SignedFixed};

use super::floor_raise::{FloorRaiseError, FloorRaiseReceipt, verify_raise};
use crate::state::Market;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaiseFloorPreservingAreaArgs<N: SignedFixed> {
    pub floor_increase_by: Pos<N>,
    pub new_ramp_end: NonNeg<N>,
    /// Exact proposed width; zero is valid.
    pub new_ramp_width: NonNeg<N>,
    /// Maximum upward gap at the floor/ramp join, in normalized price units.
    pub max_floor_join_gap: NonNeg<N>,
    /// Maximum decrease in backing, in normalized reserve units. Growth is forbidden.
    pub max_area_decrease: NonNeg<N>,
    pub market: Market<N>,
}

/// Raise the floor by redistributing existing backing. Any numerical loss must
/// fit the caller's allowance; the entire area-change enclosure must be <= 0.
/// Reserves, supply, main placement, and fee settings remain unchanged.
pub fn raise_floor_preserving_area<N: SignedFixed>(
    args: RaiseFloorPreservingAreaArgs<N>,
    characteristic: &dyn CharacteristicEngine<N>,
) -> Result<FloorRaiseReceipt<N>, FloorRaiseError> {
    let area_allowance = args
        .max_area_decrease
        .get()
        .checked_neg()
        .map_err(|_| FloorRaiseError::ArithmeticFailure)?;
    verify_raise(
        args.market,
        FloorRaiseProposal {
            floor_increase_by: args.floor_increase_by,
            ramp_end: args.new_ramp_end,
            ramp_width: args.new_ramp_width,
        },
        FloorRaisePolicy::new(args.max_floor_join_gap, area_allowance),
        characteristic,
    )
}

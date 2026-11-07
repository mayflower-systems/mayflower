use mayf_avm_calculator::{CharacteristicEngine, FloorRaisePolicy, FloorRaiseProposal};
use mayf_fixed_number::{NonNeg, Pos, SignedFixed};

use super::floor_raise::{FloorRaiseError, FloorRaiseReceipt, verify_raise};
use crate::state::Market;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaiseFloorFromSurplusArgs<N: SignedFixed> {
    pub floor_increase_by: Pos<N>,
    pub new_ramp_end: NonNeg<N>,
    /// Exact proposed width; zero is valid.
    pub new_ramp_width: NonNeg<N>,
    /// Maximum upward gap at the floor/ramp join, in normalized price units.
    pub max_floor_join_gap: NonNeg<N>,
    /// Maximum increase in backing, in normalized reserve units. Shrinkage is forbidden.
    pub max_area_increase: NonNeg<N>,
    pub market: Market<N>,
}

/// Raise the floor using existing surplus, bounded by the caller's reserve
/// budget. The new backing upper bound must be covered by held reserves and
/// outstanding advances. This action neither transfers reserves nor charges fees.
pub fn raise_floor_from_surplus<N: SignedFixed>(
    args: RaiseFloorFromSurplusArgs<N>,
    characteristic: &dyn CharacteristicEngine<N>,
) -> Result<FloorRaiseReceipt<N>, FloorRaiseError> {
    verify_raise(
        args.market,
        FloorRaiseProposal {
            floor_increase_by: args.floor_increase_by,
            ramp_end: args.new_ramp_end,
            ramp_width: args.new_ramp_width,
        },
        FloorRaisePolicy::new(args.max_floor_join_gap, args.max_area_increase.get()),
        characteristic,
    )
}

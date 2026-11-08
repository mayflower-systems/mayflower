use mayf_fixed_number::FixedNumber;

use crate::Lane;
use crate::tree::LanePool;
use crate::weighting::LaneGauge;

/// Registering a lane needs no inputs.
pub struct RegisterLaneArgs;

pub struct RegisterLaneReceipt<N> {
    pub new_lane: Lane<N>,
}

/// Register a lane with no votes and no balance. Lanes are permanent. The protocol does not
/// change, because a lane earns nothing until it has votes.
pub fn register_lane<N>(_args: RegisterLaneArgs) -> RegisterLaneReceipt<N>
where
    N: FixedNumber,
{
    let new_lane = Lane {
        gauge: LaneGauge::default(),
        pool: LanePool::default(),
    };

    RegisterLaneReceipt { new_lane }
}

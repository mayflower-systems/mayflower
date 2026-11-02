use mayf_fixed_number::{AboveOne, FixedNumber, NonNeg};
use mayf_micro_basis_points::MicroBasisPoints;

pub struct Market<N>
where
    N: FixedNumber,
{
    pub segmentation_geometry: SegmentationGeometry<N>,

    /// the total supply of AVA  by the market
    pub supply_ava: u64,

    /// The amount of reserve actively held by the market
    pub reserve_held: u64,

    /// The amount of reserve that has been advanced from the market
    pub reserve_advanced: u64,

    pub fee_rates: FeeRates,

    pub protocol_fee_rate: MicroBasisPoints,
}

pub struct SegmentationGeometry<N>
where
    N: FixedNumber,
{
    pub floor: NonNeg<N>,
    /// The width of the ramp
    /// ramp_start = ramp_end - ramp_width
    /// treated as rational number so that the calculated ramp_start (and ramp slope) can be precise for the sake of rounding errors
    pub ramp_width: NonNeg<N>,
    pub ramp_scalar: AboveOne<N>,
    pub ramp_end: u64,
}

/// All fees are taken from the reserve asset
pub struct FeeRates {
    pub issue_fee: MicroBasisPoints,
    pub redeem_fee: MicroBasisPoints,
    pub exercise_fee: MicroBasisPoints,
    pub reserve_access_fee: MicroBasisPoints,
}

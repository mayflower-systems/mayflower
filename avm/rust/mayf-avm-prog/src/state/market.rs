use mayf_avm_calculator::MarketState;
use mayf_fixed_number::{AboveOne, FixedNumber, NonNeg};
use mayf_micro_basis_points::MicroBasisPoints;

/// A market snapshot in normalized AVA and reserve quantities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Market<N: FixedNumber> {
    /// The calculator owns segmentation and the progress/circulation relationship.
    pub calculator_state: MarketState<N>,
    /// Immutable schedule configuration, bound to the same characteristic engine.
    pub ramp_scalar: AboveOne<N>,
    /// Liquid reserve available for payouts.
    pub reserve_held: NonNeg<N>,
    /// Reserve advanced against positions and still owed to the market.
    pub reserve_advanced: NonNeg<N>,
    pub fee_rates: FeeRates,
    pub protocol_fee_rate: MicroBasisPoints,
}

/// All fees are denominated in the reserve asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeeRates {
    pub issue_fee: MicroBasisPoints,
    pub redeem_fee: MicroBasisPoints,
    pub exercise_fee: MicroBasisPoints,
    pub reserve_access_fee: MicroBasisPoints,
}

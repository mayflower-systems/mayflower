use mayf_avm_calculator::AvmCalculator;
use mayf_fixed_number::SignedFixed;

pub use super::redeem_ava::{
    RedeemAvaArgs as RedeemAtFloorArgs, RedeemAvaError as RedeemAtFloorError,
    RedeemAvaReceipt as RedeemAtFloorReceipt,
};

/// Redeem against the floor tail while preserving progress and segmentation.
pub fn redeem_at_floor<N: SignedFixed>(
    args: RedeemAtFloorArgs<N>,
) -> Result<RedeemAtFloorReceipt<N>, RedeemAtFloorError> {
    let position_next = args.remaining_position()?;
    let quote = AvmCalculator::new(args.market.ramp_scalar)
        .quote_redeem_at_floor(&args.market.calculator_state, args.amount_ava)
        .map_err(RedeemAtFloorError::Calculator)?;
    args.check_collateral(&position_next)?;
    args.settle(position_next, quote)
}

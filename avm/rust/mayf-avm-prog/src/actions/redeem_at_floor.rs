use crate::{
    fees::{FeeChain, split_fee},
    state::{Market, Position},
};
use mayf_fixed_number::{FixedNumber, IntegralRounding};

pub struct RedeemAtFloorArgs<N>
where
    N: FixedNumber,
{
    pub amount_ava: u64,
    /// The minimum expected amount of reserve received after fees
    pub min_reserve_out: u64,
    pub market: Market<N>,
    pub position: Position,
}

pub struct RedeemAtFloorReceipt<N>
where
    N: FixedNumber,
{
    /// the next state of the market
    pub market_next: Market<N>,
    /// the next state of the position
    pub position_next: Position,
    /// Reserve paid to the subject after fees.
    pub reserve_to_subject: u64,
    /// the fees that were paid
    pub fee_chain: FeeChain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedeemAtFloorError {
    AvaBalanceTooLow,
    SupplyTooLow,
    FloorLiquidityTooLow,
    InvalidSegmentation,
    PositionUnderCollateralized,
    ReserveBalanceTooLow,
    MinimumReserveOutNotMet,
    ArithmeticFailure,
}

/// Burn AVA against the flat tail without changing ramp width or outstanding advances.
pub fn redeem_at_floor<N>(
    args: RedeemAtFloorArgs<N>,
) -> Result<RedeemAtFloorReceipt<N>, RedeemAtFloorError>
where
    N: IntegralRounding,
{
    let ava_owned = args
        .position
        .ava_owned
        .checked_sub(args.amount_ava)
        .ok_or(RedeemAtFloorError::AvaBalanceTooLow)?;
    let supply_ava = args
        .market
        .supply_ava
        .checked_sub(args.amount_ava)
        .ok_or(RedeemAtFloorError::SupplyTooLow)?;

    let mut segmentation_geometry = args.market.segmentation_geometry;
    if segmentation_geometry.ramp_end > args.market.supply_ava {
        return Err(RedeemAtFloorError::InvalidSegmentation);
    }
    segmentation_geometry.ramp_end = segmentation_geometry
        .ramp_end
        .checked_sub(args.amount_ava)
        .ok_or(RedeemAtFloorError::FloorLiquidityTooLow)?;
    let ramp_end = N::checked_from_u64(segmentation_geometry.ramp_end)
        .map_err(|_| RedeemAtFloorError::ArithmeticFailure)?;
    if segmentation_geometry.ramp_width.get() > ramp_end {
        return Err(RedeemAtFloorError::FloorLiquidityTooLow);
    }

    let position_next = Position {
        ava_owned,
        ..args.position
    };
    let floor = segmentation_geometry.floor;
    if !position_next
        .is_healthy(floor)
        .map_err(|_| RedeemAtFloorError::ArithmeticFailure)?
    {
        return Err(RedeemAtFloorError::PositionUnderCollateralized);
    }

    let gross_reserve = N::checked_from_u64(args.amount_ava)
        .and_then(|amount| amount.checked_mul_floor(floor.get()))
        .and_then(N::checked_floor_to_u64)
        .map_err(|_| RedeemAtFloorError::ArithmeticFailure)?;
    let gross_fee = args.market.fee_rates.redeem_fee.apply_ceil(gross_reserve);
    let reserve_to_subject = gross_reserve - gross_fee;
    if reserve_to_subject < args.min_reserve_out {
        return Err(RedeemAtFloorError::MinimumReserveOutNotMet);
    }

    // The subject and both fee recipients are paid from held reserve.
    let reserve_held = args
        .market
        .reserve_held
        .checked_sub(gross_reserve)
        .ok_or(RedeemAtFloorError::ReserveBalanceTooLow)?;
    let fee_chain = split_fee(gross_fee, args.market.protocol_fee_rate);
    let market_next = Market {
        segmentation_geometry,
        supply_ava,
        reserve_held,
        ..args.market
    };

    Ok(RedeemAtFloorReceipt {
        market_next,
        position_next,
        reserve_to_subject,
        fee_chain,
    })
}

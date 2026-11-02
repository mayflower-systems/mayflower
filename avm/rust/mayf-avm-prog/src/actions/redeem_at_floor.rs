use crate::{
    fees::FeeChain,
    state::{Market, Position},
};
use mayf_fixed_number::FixedNumber;

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
    /// the amount of reserve that was advanced to the subject
    pub reserve_to_subject: u64,
    /// the fees that were paid
    pub fee_chain: FeeChain,
}

pub enum RedeemAtFloorError {
    AvaBalanceTooLow,
    PositionUnderCollateralized,
}

pub fn redeem_at_floor<N>(
    args: RedeemAtFloorArgs<N>,
) -> Result<RedeemAtFloorReceipt<N>, RedeemAtFloorError>
where
    N: FixedNumber,
{
    let Market {
        segmentation_geometry,
        supply_ava,
        reserve_held,
        reserve_advanced,
        fee_rates,
        protocol_fee_rate,
    } = args.market;

    let Position {
        ava_owned,
        reserve_advanced: _,
    } = args.position;

    // gross reserve out = amount_ava * floor price
    // take redeem fee
    // take gross fee and split between market owner and protocol

    // create receipt
    // - supply of AVA goes down
    // - AVA balance in position goes down
    // - the ramp has a horizontal shift to the left

    todo!()
}

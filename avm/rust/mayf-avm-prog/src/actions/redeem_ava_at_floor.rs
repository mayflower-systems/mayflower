use mayf_avm_calculator::AvmCalculator;
use mayf_fixed_number::{FixedNumber, NonNeg, Pos, SignedFixed};

use crate::fees::{FeeChain, apply_fee, split_fee};
use crate::state::{Market, Position};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedeemAtFloorArgs<N: FixedNumber> {
    pub amount_ava: Pos<N>,
    /// Minimum normalized reserve received after fees.
    pub min_reserve_out: NonNeg<N>,
    pub market: Market<N>,
    pub position: Position<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedeemAtFloorReceipt<N: FixedNumber> {
    pub market_next: Market<N>,
    pub position_next: Position<N>,
    /// Normalized reserve paid to the subject after fees.
    pub reserve_to_subject: NonNeg<N>,
    pub fee_chain: FeeChain<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedeemAtFloorError {
    AvaBalanceTooLow,
    Calculator(mayf_avm_calculator::Error),
    PositionUnderCollateralized,
    ReserveBalanceTooLow,
    MinimumReserveOutNotMet,
    ArithmeticFailure,
}

/// Redeem against the floor tail while preserving progress and segmentation.
pub fn redeem_at_floor<N: SignedFixed>(
    args: RedeemAtFloorArgs<N>,
) -> Result<RedeemAtFloorReceipt<N>, RedeemAtFloorError> {
    if NonNeg::from(args.amount_ava) > args.position.ava_owned {
        return Err(RedeemAtFloorError::AvaBalanceTooLow);
    }
    let ava_owned = args
        .position
        .ava_owned
        .checked_sub(args.amount_ava)
        .map_err(|_| RedeemAtFloorError::ArithmeticFailure)?;
    let quote = AvmCalculator::new(args.market.ramp_scalar)
        .quote_redeem_at_floor(&args.market.calculator_state, args.amount_ava)
        .map_err(RedeemAtFloorError::Calculator)?;

    let position_next = Position {
        ava_owned,
        ..args.position
    };
    let floor = args.market.calculator_state.segmentation().floor();
    if !position_next
        .is_healthy(floor)
        .map_err(|_| RedeemAtFloorError::ArithmeticFailure)?
    {
        return Err(RedeemAtFloorError::PositionUnderCollateralized);
    }

    let gross_reserve = quote.reserve.lower();
    let fee = apply_fee(gross_reserve, args.market.fee_rates.redeem_fee)
        .map_err(|_| RedeemAtFloorError::ArithmeticFailure)?;
    let reserve_to_subject = gross_reserve
        .checked_sub(fee)
        .map_err(|_| RedeemAtFloorError::ArithmeticFailure)?;
    if reserve_to_subject < args.min_reserve_out {
        return Err(RedeemAtFloorError::MinimumReserveOutNotMet);
    }
    if gross_reserve > args.market.reserve_held {
        return Err(RedeemAtFloorError::ReserveBalanceTooLow);
    }
    let reserve_held = args
        .market
        .reserve_held
        .checked_sub(gross_reserve)
        .map_err(|_| RedeemAtFloorError::ArithmeticFailure)?;
    let fee_chain = split_fee(fee, args.market.protocol_fee_rate)
        .map_err(|_| RedeemAtFloorError::ArithmeticFailure)?;

    Ok(RedeemAtFloorReceipt {
        market_next: Market {
            calculator_state: quote.next_state,
            reserve_held,
            ..args.market
        },
        position_next,
        reserve_to_subject,
        fee_chain,
    })
}

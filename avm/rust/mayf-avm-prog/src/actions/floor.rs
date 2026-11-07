//! Floor-priced supply changes and their reserve settlement.

use mayf_avm_calculator::AvmCalculator;
use mayf_fixed_number::{FixedNumber, NonNeg, Pos, SignedFixed};

use crate::fees::{FeeChain, apply_fee, split_fee};
use crate::state::{Market, Position};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IssueAvaAtFloorArgs<N: FixedNumber> {
    pub amount_ava: Pos<N>,
    /// Maximum normalized reserve the holder will pay, including the exercise fee.
    pub max_reserve_in: NonNeg<N>,
    pub market: Market<N>,
    pub position: Position<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IssueAvaAtFloorReceipt<N: FixedNumber> {
    pub market_next: Market<N>,
    pub position_next: Position<N>,
    /// Total normalized reserve owed by the holder, including both fee shares.
    pub reserve_from_subject: NonNeg<N>,
    pub fee_chain: FeeChain<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueAvaAtFloorError {
    Calculator(mayf_avm_calculator::Error),
    MaximumReserveInExceeded,
    ArithmeticFailure,
}

/// Issue AVA at the current floor for an authorized option exercise.
/// The new AVA extends the floor tail and preserves characteristic progress.
pub fn issue_ava_at_floor<N: SignedFixed>(
    args: IssueAvaAtFloorArgs<N>,
) -> Result<IssueAvaAtFloorReceipt<N>, IssueAvaAtFloorError> {
    let ava_owned = args
        .position
        .ava_owned
        .checked_add(args.amount_ava)
        .map_err(|_| IssueAvaAtFloorError::ArithmeticFailure)?;
    let quote = AvmCalculator::new(args.market.ramp_scalar)
        .quote_issue_at_floor(&args.market.calculator_state, args.amount_ava)
        .map_err(IssueAvaAtFloorError::Calculator)?;

    let reserve_to_market = quote.reserve.upper();
    let fee = apply_fee(reserve_to_market, args.market.fee_rates.exercise_fee)
        .map_err(|_| IssueAvaAtFloorError::ArithmeticFailure)?;
    let reserve_from_subject = reserve_to_market
        .checked_add(fee)
        .map_err(|_| IssueAvaAtFloorError::ArithmeticFailure)?;
    if reserve_from_subject > args.max_reserve_in {
        return Err(IssueAvaAtFloorError::MaximumReserveInExceeded);
    }

    let fee_chain = split_fee(fee, args.market.protocol_fee_rate)
        .map_err(|_| IssueAvaAtFloorError::ArithmeticFailure)?;
    let reserve_held = args
        .market
        .reserve_held
        .checked_add(reserve_to_market)
        .map_err(|_| IssueAvaAtFloorError::ArithmeticFailure)?;

    Ok(IssueAvaAtFloorReceipt {
        market_next: Market {
            calculator_state: quote.next_state,
            reserve_held,
            ..args.market
        },
        position_next: Position {
            ava_owned,
            ..args.position
        },
        reserve_from_subject,
        fee_chain,
    })
}

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

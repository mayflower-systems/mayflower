use core::cmp::Ordering;

use mayf_avm_calculator::{AvmCalculator, CharacteristicEngine, MarketState};
use mayf_fixed_number::{FixedNumber, NonNeg, Pos, SignedFixed};

use crate::fees::{FeeChain, apply_fee, split_fee};
use crate::state::{Market, Position};

/// Minimum wallet balance after the net reserve transfer and all fees.
/// A minimum below the initial balance caps spending. A higher minimum requires a payout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlippageProtection<N: FixedNumber> {
    pub subject_initial_reserve_balance: NonNeg<N>,
    pub subject_minimum_final_reserve_balance: NonNeg<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReconcilePositionArgs<N: FixedNumber> {
    pub target_position_ava_owned: NonNeg<N>,
    pub target_position_reserve_advanced: NonNeg<N>,
    pub slippage_protection: SlippageProtection<N>,
    pub market: Market<N>,
    pub position: Position<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReconcilePositionReceipt<N: FixedNumber> {
    pub market_next: Market<N>,
    pub position_next: Position<N>,
    /// Signed net reserve transfer after fees. Positive pays the subject, negative debits them.
    pub reserve_to_subject: N,
    /// Fees on newly drawn reserve. Repayment carries no fee.
    pub borrow_fee_chain: FeeChain<N>,
    /// Fees on the AVA purchase or redemption.
    pub trade_fee_chain: FeeChain<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconcilePositionError {
    Calculator(mayf_avm_calculator::Error),
    PositionExceedsMarket,
    PositionUnderCollateralized,
    ReserveBalanceTooLow,
    SubjectReserveBalanceTooLow,
    MinimumReserveBalanceNotMet,
    ArithmeticFailure,
}

/// Reconcile to exact position balances, checking collateral and liquidity on
/// the combined result. The host settles the net transfer, fees, and state atomically.
pub fn reconcile_position<N: SignedFixed>(
    args: ReconcilePositionArgs<N>,
    characteristic: &dyn CharacteristicEngine<N>,
) -> Result<ReconcilePositionReceipt<N>, ReconcilePositionError> {
    let position_next = checked_target(&args)?;
    let advance = plan_advance(&args)?;
    let trade = quote_trade(&args, characteristic)?;
    settle(&args, position_next, advance, trade)
}

fn checked_target<N: SignedFixed>(
    args: &ReconcilePositionArgs<N>,
) -> Result<Position<N>, ReconcilePositionError> {
    if args.position.ava_owned
        > args
            .market
            .calculator_state
            .accounting()
            .circulating_supply()
        || args.position.reserve_advanced > args.market.reserve_advanced
    {
        return Err(ReconcilePositionError::PositionExceedsMarket);
    }
    let target = Position {
        ava_owned: args.target_position_ava_owned,
        reserve_advanced: args.target_position_reserve_advanced,
    };
    if !target
        .is_healthy(args.market.calculator_state.segmentation().floor())
        .map_err(|_| ReconcilePositionError::ArithmeticFailure)?
    {
        return Err(ReconcilePositionError::PositionUnderCollateralized);
    }
    Ok(target)
}

struct AdvancePlan<N: FixedNumber> {
    market_advanced: NonNeg<N>,
    draw: N,
    fee: NonNeg<N>,
}

fn plan_advance<N: SignedFixed>(
    args: &ReconcilePositionArgs<N>,
) -> Result<AdvancePlan<N>, ReconcilePositionError> {
    let market_advanced = args
        .market
        .reserve_advanced
        .checked_sub(args.position.reserve_advanced)
        .and_then(|others| others.checked_add(args.target_position_reserve_advanced))
        .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
    let draw = args
        .target_position_reserve_advanced
        .get()
        .checked_sub(args.position.reserve_advanced.get())
        .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
    let newly_drawn =
        NonNeg::new(draw.max(N::ZERO)).map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
    let fee = apply_fee(newly_drawn, args.market.fee_rates.reserve_access_fee)
        .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
    Ok(AdvancePlan {
        market_advanced,
        draw,
        fee,
    })
}

struct TradePlan<N: FixedNumber> {
    state_next: MarketState<N>,
    reserve_to_market: N,
    fee: NonNeg<N>,
}

fn quote_trade<N: SignedFixed>(
    args: &ReconcilePositionArgs<N>,
    characteristic: &dyn CharacteristicEngine<N>,
) -> Result<TradePlan<N>, ReconcilePositionError> {
    let calculator = AvmCalculator::new(args.market.ramp_scalar);
    let state = &args.market.calculator_state;
    match args.target_position_ava_owned.cmp(&args.position.ava_owned) {
        Ordering::Greater => {
            let amount = args
                .target_position_ava_owned
                .checked_sub(args.position.ava_owned)
                .and_then(Pos::try_from)
                .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
            let quote = calculator
                .quote_issue(state, amount, characteristic)
                .map_err(ReconcilePositionError::Calculator)?;
            let reserve = quote.reserve.upper();
            let fee = apply_fee(reserve, args.market.fee_rates.issue_fee)
                .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
            Ok(TradePlan {
                state_next: quote.next_state,
                reserve_to_market: reserve.get(),
                fee,
            })
        }
        Ordering::Less => {
            let amount = args
                .position
                .ava_owned
                .checked_sub(args.target_position_ava_owned)
                .and_then(Pos::try_from)
                .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
            let quote = calculator
                .quote_redeem(state, amount, characteristic)
                .map_err(ReconcilePositionError::Calculator)?;
            let reserve = quote.reserve.lower();
            let fee = apply_fee(reserve, args.market.fee_rates.redeem_fee)
                .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
            let reserve_to_market = reserve
                .get()
                .checked_neg()
                .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
            Ok(TradePlan {
                state_next: quote.next_state,
                reserve_to_market,
                fee,
            })
        }
        Ordering::Equal => Ok(TradePlan {
            state_next: *state,
            reserve_to_market: N::ZERO,
            fee: NonNeg::zero(),
        }),
    }
}

fn settle<N: SignedFixed>(
    args: &ReconcilePositionArgs<N>,
    position_next: Position<N>,
    advance: AdvancePlan<N>,
    trade: TradePlan<N>,
) -> Result<ReconcilePositionReceipt<N>, ReconcilePositionError> {
    // Net the internal purchase/sale and draw/repayment before applying balances.
    let market_change = trade
        .reserve_to_market
        .checked_sub(advance.draw)
        .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
    let reserve_held = args
        .market
        .reserve_held
        .get()
        .checked_add(market_change)
        .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
    let reserve_held =
        NonNeg::new(reserve_held).map_err(|_| ReconcilePositionError::ReserveBalanceTooLow)?;
    // Subtract fees separately so their sum need not fit in the numeric type.
    let reserve_to_subject = market_change
        .checked_neg()
        .and_then(|amount| amount.checked_sub(trade.fee.get()))
        .and_then(|amount| amount.checked_sub(advance.fee.get()))
        .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
    args.slippage_protection.check(reserve_to_subject)?;

    Ok(ReconcilePositionReceipt {
        market_next: Market {
            calculator_state: trade.state_next,
            reserve_held,
            reserve_advanced: advance.market_advanced,
            ..args.market
        },
        position_next,
        reserve_to_subject,
        borrow_fee_chain: split_fee(advance.fee, args.market.protocol_fee_rate)
            .map_err(|_| ReconcilePositionError::ArithmeticFailure)?,
        trade_fee_chain: split_fee(trade.fee, args.market.protocol_fee_rate)
            .map_err(|_| ReconcilePositionError::ArithmeticFailure)?,
    })
}

impl<N: SignedFixed> SlippageProtection<N> {
    fn check(&self, reserve_to_subject: N) -> Result<(), ReconcilePositionError> {
        let final_balance = self
            .subject_initial_reserve_balance
            .get()
            .checked_add(reserve_to_subject)
            .map_err(|_| ReconcilePositionError::ArithmeticFailure)?;
        if final_balance < N::ZERO {
            return Err(ReconcilePositionError::SubjectReserveBalanceTooLow);
        }
        if final_balance < self.subject_minimum_final_reserve_balance.get() {
            return Err(ReconcilePositionError::MinimumReserveBalanceNotMet);
        }
        Ok(())
    }
}

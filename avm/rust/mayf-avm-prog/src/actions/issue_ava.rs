use mayf_avm_calculator::{AvmCalculator, CharacteristicEngine, TradeQuote};
use mayf_fixed_number::{FixedNumber, NonNeg, Pos, SignedFixed};
use mayf_micro_basis_points::MicroBasisPoints;

use crate::fees::{FeeChain, apply_fee, split_fee};
use crate::state::{Market, Position};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IssueAvaArgs<N: FixedNumber> {
    pub amount_ava: Pos<N>,
    /// Maximum normalized reserve the buyer will pay, including fees.
    pub max_reserve_in: NonNeg<N>,
    pub market: Market<N>,
    pub position: Position<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IssueAvaReceipt<N: FixedNumber> {
    pub market_next: Market<N>,
    pub position_next: Position<N>,
    /// Total normalized reserve owed by the buyer, including both fee shares.
    pub reserve_from_subject: NonNeg<N>,
    pub fee_chain: FeeChain<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueAvaError {
    Calculator(mayf_avm_calculator::Error),
    MaximumReserveInExceeded,
    ArithmeticFailure,
}

/// Quote an exact AVA purchase and return the balances to commit on settlement.
pub fn issue_ava<N: SignedFixed>(
    args: IssueAvaArgs<N>,
    characteristic: &dyn CharacteristicEngine<N>,
) -> Result<IssueAvaReceipt<N>, IssueAvaError> {
    let position_next = args.credited_position()?;
    let quote = AvmCalculator::new(args.market.ramp_scalar)
        .quote_issue(
            &args.market.calculator_state,
            args.amount_ava,
            characteristic,
        )
        .map_err(IssueAvaError::Calculator)?;

    args.settle(position_next, quote, args.market.fee_rates.issue_fee)
}

impl<N: SignedFixed> IssueAvaArgs<N> {
    pub(super) fn credited_position(&self) -> Result<Position<N>, IssueAvaError> {
        let ava_owned = self
            .position
            .ava_owned
            .checked_add(self.amount_ava)
            .map_err(|_| IssueAvaError::ArithmeticFailure)?;
        Ok(Position {
            ava_owned,
            ..self.position
        })
    }

    pub(super) fn settle(
        &self,
        position_next: Position<N>,
        quote: TradeQuote<N>,
        fee_rate: MicroBasisPoints,
    ) -> Result<IssueAvaReceipt<N>, IssueAvaError> {
        let reserve_to_market = quote.reserve.upper();
        let fee =
            apply_fee(reserve_to_market, fee_rate).map_err(|_| IssueAvaError::ArithmeticFailure)?;
        let reserve_from_subject = reserve_to_market
            .checked_add(fee)
            .map_err(|_| IssueAvaError::ArithmeticFailure)?;
        if reserve_from_subject > self.max_reserve_in {
            return Err(IssueAvaError::MaximumReserveInExceeded);
        }

        let fee_chain = split_fee(fee, self.market.protocol_fee_rate)
            .map_err(|_| IssueAvaError::ArithmeticFailure)?;
        let reserve_held = self
            .market
            .reserve_held
            .checked_add(reserve_to_market)
            .map_err(|_| IssueAvaError::ArithmeticFailure)?;

        Ok(IssueAvaReceipt {
            market_next: Market {
                calculator_state: quote.next_state,
                reserve_held,
                ..self.market
            },
            position_next,
            reserve_from_subject,
            fee_chain,
        })
    }
}

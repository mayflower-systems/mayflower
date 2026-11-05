use mayf_avm_calculator::{AvmCalculator, CharacteristicEngine, TradeQuote};
use mayf_fixed_number::{FixedNumber, NonNeg, Pos, SignedFixed};

use crate::fees::{FeeChain, apply_fee, split_fee};
use crate::state::{Market, Position};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedeemAvaArgs<N: FixedNumber> {
    pub amount_ava: Pos<N>,
    /// Minimum normalized reserve received after fees.
    pub min_reserve_out: NonNeg<N>,
    pub market: Market<N>,
    pub position: Position<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedeemAvaReceipt<N: FixedNumber> {
    pub market_next: Market<N>,
    pub position_next: Position<N>,
    /// Normalized reserve paid to the subject after fees.
    pub reserve_to_subject: NonNeg<N>,
    pub fee_chain: FeeChain<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedeemAvaError {
    AvaBalanceTooLow,
    Calculator(mayf_avm_calculator::Error),
    PositionUnderCollateralized,
    ReserveBalanceTooLow,
    MinimumReserveOutNotMet,
    ArithmeticFailure,
}

/// Sell an exact AVA amount on the current schedule. The quote prices the
/// entire sale before contraction and supplies the geometry for the next trade.
pub fn redeem_ava<N: SignedFixed>(
    args: RedeemAvaArgs<N>,
    characteristic: &dyn CharacteristicEngine<N>,
) -> Result<RedeemAvaReceipt<N>, RedeemAvaError> {
    let position_next = args.remaining_position()?;
    args.check_collateral(&position_next)?;
    let quote = AvmCalculator::new(args.market.ramp_scalar)
        .quote_redeem(
            &args.market.calculator_state,
            args.amount_ava,
            characteristic,
        )
        .map_err(RedeemAvaError::Calculator)?;

    args.settle(position_next, quote)
}

impl<N: SignedFixed> RedeemAvaArgs<N> {
    pub(super) fn remaining_position(&self) -> Result<Position<N>, RedeemAvaError> {
        if NonNeg::from(self.amount_ava) > self.position.ava_owned {
            return Err(RedeemAvaError::AvaBalanceTooLow);
        }
        let ava_owned = self
            .position
            .ava_owned
            .checked_sub(self.amount_ava)
            .map_err(|_| RedeemAvaError::ArithmeticFailure)?;
        Ok(Position {
            ava_owned,
            ..self.position
        })
    }

    pub(super) fn check_collateral(&self, position: &Position<N>) -> Result<(), RedeemAvaError> {
        let floor = self.market.calculator_state.segmentation().floor();
        if !position
            .is_healthy(floor)
            .map_err(|_| RedeemAvaError::ArithmeticFailure)?
        {
            return Err(RedeemAvaError::PositionUnderCollateralized);
        }
        Ok(())
    }

    pub(super) fn settle(
        &self,
        position_next: Position<N>,
        quote: TradeQuote<N>,
    ) -> Result<RedeemAvaReceipt<N>, RedeemAvaError> {
        let gross_reserve = quote.reserve.lower();
        let gross_fee = apply_fee(gross_reserve, self.market.fee_rates.redeem_fee)
            .map_err(|_| RedeemAvaError::ArithmeticFailure)?;
        let reserve_to_subject = gross_reserve
            .checked_sub(gross_fee)
            .map_err(|_| RedeemAvaError::ArithmeticFailure)?;
        if reserve_to_subject < self.min_reserve_out {
            return Err(RedeemAvaError::MinimumReserveOutNotMet);
        }
        if gross_reserve > self.market.reserve_held {
            return Err(RedeemAvaError::ReserveBalanceTooLow);
        }
        let reserve_held = self
            .market
            .reserve_held
            .checked_sub(gross_reserve)
            .map_err(|_| RedeemAvaError::ArithmeticFailure)?;
        let fee_chain = split_fee(gross_fee, self.market.protocol_fee_rate)
            .map_err(|_| RedeemAvaError::ArithmeticFailure)?;

        Ok(RedeemAvaReceipt {
            market_next: Market {
                calculator_state: quote.next_state,
                reserve_held,
                ..self.market
            },
            position_next,
            reserve_to_subject,
            fee_chain,
        })
    }
}

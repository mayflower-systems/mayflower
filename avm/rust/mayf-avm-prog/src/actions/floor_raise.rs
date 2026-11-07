use mayf_avm_calculator::{
    AvmCalculator, CharacteristicEngine, FloorRaisePolicy, FloorRaiseProposal,
};
use mayf_fixed_number::{Bounds, NonNeg, SignedFixed};

use crate::state::Market;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloorRaiseReceipt<N: SignedFixed> {
    pub market_next: Market<N>,
    /// Change in required backing, in normalized reserve units.
    pub backing_change: Bounds<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorRaiseError {
    Calculator(mayf_avm_calculator::Error),
    InsufficientReserves,
    ArithmeticFailure,
}

pub(super) fn verify_raise<N: SignedFixed>(
    market: Market<N>,
    proposal: FloorRaiseProposal<N>,
    policy: FloorRaisePolicy<N>,
    characteristic: &dyn CharacteristicEngine<N>,
) -> Result<FloorRaiseReceipt<N>, FloorRaiseError> {
    let quote = AvmCalculator::new(market.ramp_scalar)
        .verify_floor_raise(&market.calculator_state, proposal, policy, characteristic)
        .map_err(FloorRaiseError::Calculator)?;
    check_backing(&market, quote.backing.upper())?;

    Ok(FloorRaiseReceipt {
        market_next: Market {
            calculator_state: quote.next_state,
            ..market
        },
        backing_change: quote.backing_change,
    })
}

fn check_backing<N: SignedFixed>(
    market: &Market<N>,
    required: NonNeg<N>,
) -> Result<(), FloorRaiseError> {
    // Advances remain backing assets. Compare without forming a potentially
    // overflowing sum of held and advanced reserves.
    if required > market.reserve_advanced {
        let required_held = required
            .checked_sub(market.reserve_advanced)
            .map_err(|_| FloorRaiseError::ArithmeticFailure)?;
        if required_held > market.reserve_held {
            return Err(FloorRaiseError::InsufficientReserves);
        }
    }
    Ok(())
}

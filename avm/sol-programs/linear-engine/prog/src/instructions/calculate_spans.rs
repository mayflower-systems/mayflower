use anchor_lang::prelude::*;
use mayf_avm_calculator::{CharacteristicEngine, LocalCharacteristic};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::{Bounds, NonNeg};

use crate::{
    error::EngineError,
    state::LinearCharacteristicParam,
    wire::{MAX_BATCH_SPANS, SpanRequest, SpanResponse},
};

#[derive(Accounts)]
pub struct CalculateSpansAccounts<'info> {
    pub linear_characteristic_param: Account<'info, LinearCharacteristicParam>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CalculateSpansArgs {
    pub spans: Vec<SpanRequest>,
}

pub fn calculate_spans_handler(
    ctx: Context<CalculateSpansAccounts>,
    args: CalculateSpansArgs,
) -> Result<Vec<SpanResponse>> {
    evaluate_spans(
        ctx.accounts.linear_characteristic_param.slope_q64x64,
        &args.spans,
    )
}

fn evaluate_spans(slope_q64x64: u128, spans: &[SpanRequest]) -> Result<Vec<SpanResponse>> {
    require!(spans.len() <= MAX_BATCH_SPANS, EngineError::BatchTooLarge);
    let characteristic = LinearCharacteristic::try_new(U64F64::from_scaled_u128(slope_q64x64))
        .map_err(|_| error!(EngineError::InvalidSlope))?;
    let engine = LocalCharacteristic::new(characteristic);
    let mut responses = Vec::with_capacity(spans.len());
    for span in spans {
        let request = (*span).into();
        let zero = Bounds::exact(NonNeg::zero());
        let mut response = mayf_avm_calculator::SpanResponse {
            y_at_lo: zero,
            y_at_hi: zero,
            area: zero,
        };
        engine
            .spans(
                core::slice::from_ref(&request),
                core::slice::from_mut(&mut response),
            )
            .map_err(|_| error!(EngineError::ArithmeticFailure))?;
        responses.push(response.into());
    }
    Ok(responses)
}

#[cfg(test)]
mod tests;

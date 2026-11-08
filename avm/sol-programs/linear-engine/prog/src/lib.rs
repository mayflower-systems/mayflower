#![allow(unexpected_cfgs)]
#![doc = include_str!("../README.md")]

use anchor_lang::prelude::*;
pub mod error;
mod instructions;
pub mod state;
pub mod wire;

#[cfg(feature = "env_local")]
declare_id!("AWzdC1NqcyWzTPtmKBR9vWdkJPsgML4F2oDu5fMCCr9r");

// TODO: Update to production address
#[cfg(feature = "env_prod")]
declare_id!("AWzdC1NqcyWzTPtmKBR9vWdkJPsgML4F2oDu5fMCCr9r");

pub use instructions::*;
use wire::SpanResponse;

#[program]
pub mod mayf_avm_engine_linear_sol {
    use super::*;

    pub fn create_linear_engine(
        ctx: Context<CreateLinearEngineAccounts>,
        args: CreateLinearEngineArgs,
    ) -> Result<()> {
        create_linear_engine_handler(ctx, args)
    }

    /// Evaluate neutral characteristic spans and return their bounds in order.
    pub fn calculate_spans(
        ctx: Context<CalculateSpansAccounts>,
        args: CalculateSpansArgs,
    ) -> Result<Vec<SpanResponse>> {
        calculate_spans_handler(ctx, args)
    }
}

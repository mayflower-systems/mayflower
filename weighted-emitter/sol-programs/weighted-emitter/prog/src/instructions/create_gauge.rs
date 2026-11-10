use anchor_lang::prelude::*;
use mayf_weighted_emitter::{CreateGaugeArgs, create_gauge};

use crate::{
    clock::now_unix_seconds,
    error::WeightedEmitterProgramError,
    state::{emitter::Emitter, gauge::Gauge},
};

#[derive(Accounts)]
pub struct CreateGaugeAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority
    )]
    pub emitter: Account<'info, Emitter>,
    #[account(
        init,
        payer = payer,
        space = 8 + Gauge::INIT_SPACE
    )]
    pub gauge: Account<'info, Gauge>,
    pub system_program: Program<'info, System>,
}

pub fn create_gauge_handler(ctx: Context<CreateGaugeAccounts>) -> Result<()> {
    let receipt = create_gauge(CreateGaugeArgs {
        emitter: ctx.accounts.emitter.to_library(ctx.accounts.emitter.key()),
        id: ctx.accounts.gauge.key(),
        at: now_unix_seconds()?,
    })
    .map_err(WeightedEmitterProgramError::from)?;
    ctx.accounts.emitter.apply_library(receipt.emitter_next)?;
    ctx.accounts.gauge.emitter = ctx.accounts.emitter.key();
    ctx.accounts.gauge.total_minted = 0;
    ctx.accounts.gauge.apply_library(receipt.new_gauge);
    Ok(())
}

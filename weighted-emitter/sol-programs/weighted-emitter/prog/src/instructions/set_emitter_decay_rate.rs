use anchor_lang::prelude::*;
use mayf_weighted_emitter::{
    SetEmitterDecayRateArgs as LibSetEmitterDecayRateArgs, set_emitter_decay_rate,
};

use crate::{
    clock::now_unix_seconds,
    error::WeightedEmitterProgramError,
    state::{emitter::Emitter, numbers::rate_from_wire},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetEmitterDecayRateArgs {
    pub annual_decay_rate_q64f64: u128,
}

#[derive(Accounts)]
pub struct SetEmitterDecayRateAccounts<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority
    )]
    pub emitter: Account<'info, Emitter>,
}

pub fn set_emitter_decay_rate_handler(
    ctx: Context<SetEmitterDecayRateAccounts>,
    args: SetEmitterDecayRateArgs,
) -> Result<()> {
    let receipt = set_emitter_decay_rate(LibSetEmitterDecayRateArgs {
        emitter: ctx.accounts.emitter.to_library(ctx.accounts.emitter.key()),
        annual_decay_rate: rate_from_wire(args.annual_decay_rate_q64f64),
        at: now_unix_seconds()?,
    })
    .map_err(WeightedEmitterProgramError::from)?;
    ctx.accounts.emitter.apply_library(receipt.emitter_next)
}

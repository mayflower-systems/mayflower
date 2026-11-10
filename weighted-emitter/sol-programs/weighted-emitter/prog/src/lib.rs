#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

mod clock;
mod constants;
mod error;
mod instructions;
mod state;

use instructions::*;

declare_id!("6tdyEnBkiNYwbF5FmiF4Dq8piqq16CA642JEkMS88RgL");

#[program]
pub mod mayf_weighted_emitter_sol {
    use super::*;

    pub fn create_emitter(
        ctx: Context<CreateEmitterAccounts>,
        args: CreateEmitterArgs,
    ) -> Result<()> {
        create_emitter_handler(ctx, args)
    }

    pub fn create_gauge(ctx: Context<CreateGaugeAccounts>) -> Result<()> {
        create_gauge_handler(ctx)
    }

    pub fn create_vote_position(ctx: Context<CreateVotePositionAccounts>) -> Result<()> {
        create_vote_position_handler(ctx)
    }

    pub fn deposit_vote(ctx: Context<DepositVoteAccounts>, amount: u64) -> Result<()> {
        deposit_vote_handler(ctx, amount)
    }

    pub fn withdraw_vote(ctx: Context<WithdrawVoteAccounts>, amount: u64) -> Result<()> {
        withdraw_vote_handler(ctx, amount)
    }

    pub fn set_emitter_decay_rate(
        ctx: Context<SetEmitterDecayRateAccounts>,
        args: SetEmitterDecayRateArgs,
    ) -> Result<()> {
        set_emitter_decay_rate_handler(ctx, args)
    }

    pub fn settle_emission(ctx: Context<SettleEmissionAccounts>) -> Result<()> {
        settle_emission_handler(ctx)
    }
}

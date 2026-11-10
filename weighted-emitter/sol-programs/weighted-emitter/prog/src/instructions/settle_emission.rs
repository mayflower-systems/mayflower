use anchor_lang::{prelude::*, solana_program::program_option::COption};
use anchor_spl::token::{Mint, MintTo, Token, TokenAccount, mint_to};
use mayf_weighted_emitter::{EmitToGaugeArgs, emit_to_gauge};

use super::vote_change::validate_minted_totals;
use crate::{
    clock::now_unix_seconds,
    constants::MINT_AUTHORITY_SEED,
    error::WeightedEmitterProgramError,
    state::{emitter::Emitter, gauge::Gauge},
};

#[derive(Accounts)]
pub struct SettleEmissionAccounts<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority,
        has_one = emission_mint
    )]
    pub emitter: Account<'info, Emitter>,
    #[account(
        mut,
        has_one = emitter
    )]
    pub gauge: Account<'info, Gauge>,
    #[account(
        mut,
        constraint = emission_mint.key() != anchor_spl::token::spl_token::native_mint::ID
            @ WeightedEmitterProgramError::NativeEmissionMint,
        constraint = emission_mint.mint_authority == COption::Some(mint_authority.key())
            @ WeightedEmitterProgramError::InvalidMintAuthority
    )]
    pub emission_mint: Account<'info, Mint>,
    /// CHECK: Canonical mint authority PDA, used only to sign the mint CPI.
    #[account(
        seeds = [MINT_AUTHORITY_SEED, emitter.key().as_ref()],
        bump = emitter.mint_authority_bump[0]
    )]
    pub mint_authority: UncheckedAccount<'info>,
    /// The emitter authority chooses the recipient; its token owner is unrestricted.
    #[account(
        mut,
        token::mint = emission_mint
    )]
    pub emission_destination: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn settle_emission_handler(ctx: Context<SettleEmissionAccounts>) -> Result<()> {
    validate_minted_totals(&ctx.accounts.emitter, &ctx.accounts.gauge)?;
    let receipt = emit_to_gauge(EmitToGaugeArgs {
        emitter: ctx.accounts.emitter.to_library(ctx.accounts.emitter.key()),
        gauge: ctx.accounts.gauge.to_library(ctx.accounts.gauge.key()),
        at: now_unix_seconds()?,
    })
    .map_err(WeightedEmitterProgramError::from)?;
    // Vote changes may already have checkpointed allocation. Mint all unissued
    // allocation, not just this invocation's newly accrued receipt.emitted.
    let amount = receipt
        .gauge_next
        .total_emitted
        .checked_sub(ctx.accounts.gauge.total_minted)
        .ok_or(WeightedEmitterProgramError::InvalidState)?;
    let total_minted = ctx
        .accounts
        .emitter
        .total_minted
        .checked_add(amount)
        .ok_or(WeightedEmitterProgramError::ArithmeticOverflow)?;
    require!(
        total_minted <= receipt.emitter_next.total_emitted,
        WeightedEmitterProgramError::InvalidState
    );

    if amount != 0 {
        let emitter_key = ctx.accounts.emitter.key();
        let signer_seeds = [
            MINT_AUTHORITY_SEED,
            emitter_key.as_ref(),
            ctx.accounts.emitter.mint_authority_bump.as_ref(),
        ];
        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                MintTo {
                    mint: ctx.accounts.emission_mint.to_account_info(),
                    to: ctx.accounts.emission_destination.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
            )
            .with_signer(&[&signer_seeds]),
            amount,
        )?;
    }
    ctx.accounts.emitter.apply_library(receipt.emitter_next)?;
    ctx.accounts.emitter.total_minted = total_minted;
    ctx.accounts.gauge.total_minted = receipt.gauge_next.total_emitted;
    ctx.accounts.gauge.apply_library(receipt.gauge_next);
    Ok(())
}

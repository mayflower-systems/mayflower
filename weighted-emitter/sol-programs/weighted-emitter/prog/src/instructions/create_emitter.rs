use anchor_lang::{prelude::*, solana_program::program_option::COption};
use anchor_spl::token::Mint;
use mayf_weighted_emitter::{CreateEmitterArgs as LibCreateEmitterArgs, create_emitter};

use crate::{
    clock::now_unix_seconds,
    constants::MINT_AUTHORITY_SEED,
    error::WeightedEmitterProgramError,
    state::{emitter::Emitter, numbers::rate_from_wire},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateEmitterArgs {
    pub emission_rate_q64f64: u128,
    pub annual_decay_rate_q64f64: u128,
}

#[derive(Accounts)]
pub struct CreateEmitterAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + Emitter::INIT_SPACE
    )]
    pub emitter: Account<'info, Emitter>,
    pub vote_mint: Account<'info, Mint>,
    #[account(
        constraint = emission_mint.key() != anchor_spl::token::spl_token::native_mint::ID
            @ WeightedEmitterProgramError::NativeEmissionMint,
        constraint = emission_mint.mint_authority == COption::Some(mint_authority.key())
            @ WeightedEmitterProgramError::InvalidMintAuthority
    )]
    pub emission_mint: Account<'info, Mint>,
    /// CHECK: The canonical PDA signs only emission minting; no data is stored here.
    #[account(
        seeds = [MINT_AUTHORITY_SEED, emitter.key().as_ref()],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn create_emitter_handler(
    ctx: Context<CreateEmitterAccounts>,
    args: CreateEmitterArgs,
) -> Result<()> {
    let receipt = create_emitter(LibCreateEmitterArgs {
        id: ctx.accounts.emitter.key(),
        emission_rate: rate_from_wire(args.emission_rate_q64f64),
        annual_decay_rate: rate_from_wire(args.annual_decay_rate_q64f64),
        at: now_unix_seconds()?,
    })
    .map_err(WeightedEmitterProgramError::from)?;
    let emitter = &mut ctx.accounts.emitter;
    emitter.authority = ctx.accounts.authority.key();
    emitter.vote_mint = ctx.accounts.vote_mint.key();
    emitter.emission_mint = ctx.accounts.emission_mint.key();
    emitter.mint_authority_bump = [ctx.bumps.mint_authority];
    emitter.total_minted = 0;
    emitter.apply_library(receipt.new_emitter)
}

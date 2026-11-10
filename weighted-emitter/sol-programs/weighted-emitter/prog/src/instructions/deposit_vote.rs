use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer, transfer};

use super::vote_change::{
    VoteChange, apply_vote_change, prepare_vote_change, validate_escrow_backing,
};
use crate::{
    constants::{VOTE_ESCROW_SEED, VOTE_POSITION_SEED},
    error::WeightedEmitterProgramError,
    state::{emitter::Emitter, gauge::Gauge, vote_position::VotePosition},
};

#[derive(Accounts)]
pub struct DepositVoteAccounts<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = vote_mint
    )]
    pub emitter: Account<'info, Emitter>,
    #[account(
        mut,
        has_one = emitter
    )]
    pub gauge: Account<'info, Gauge>,
    #[account(
        mut,
        has_one = authority,
        has_one = gauge,
        has_one = vote_escrow,
        seeds = [VOTE_POSITION_SEED, gauge.key().as_ref(), authority.key().as_ref()],
        bump = vote_position.self_bump[0]
    )]
    pub vote_position: Account<'info, VotePosition>,
    pub vote_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [VOTE_ESCROW_SEED, vote_position.key().as_ref()],
        bump,
        token::mint = vote_mint,
        token::authority = vote_position,
    )]
    pub vote_escrow: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = vote_mint,
        constraint = vote_source.key() != vote_escrow.key()
            @ WeightedEmitterProgramError::DepositSourceIsEscrow
    )]
    pub vote_source: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn deposit_vote_handler(ctx: Context<DepositVoteAccounts>, amount: u64) -> Result<()> {
    let (receipt, deposited) = prepare_vote_change(
        &ctx.accounts.emitter,
        &ctx.accounts.gauge,
        &ctx.accounts.vote_position,
        VoteChange::Deposit(amount),
    )?;
    transfer(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.vote_source.to_account_info(),
                to: ctx.accounts.vote_escrow.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        amount,
    )?;
    apply_vote_change(
        &mut ctx.accounts.emitter,
        &mut ctx.accounts.gauge,
        &mut ctx.accounts.vote_position,
        receipt,
        deposited,
    )?;
    validate_escrow_backing(&mut ctx.accounts.vote_escrow, &ctx.accounts.vote_position)
}

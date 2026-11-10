use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    constants::{VOTE_ESCROW_SEED, VOTE_POSITION_SEED},
    state::{emitter::Emitter, gauge::Gauge, vote_position::VotePosition},
};

#[derive(Accounts)]
pub struct CreateVotePositionAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(has_one = vote_mint)]
    pub emitter: Account<'info, Emitter>,

    #[account(has_one = emitter)]
    pub gauge: Account<'info, Gauge>,

    #[account(
        init,
        payer = payer,
        space = 8 + VotePosition::INIT_SPACE,
        seeds = [VOTE_POSITION_SEED, gauge.key().as_ref(), authority.key().as_ref()],
        bump
    )]
    pub vote_position: Account<'info, VotePosition>,

    pub vote_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        seeds = [VOTE_ESCROW_SEED, vote_position.key().as_ref()],
        bump,
        token::mint = vote_mint,
        token::authority = vote_position
    )]
    pub vote_escrow: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

pub fn create_vote_position_handler(ctx: Context<CreateVotePositionAccounts>) -> Result<()> {
    ctx.accounts.vote_position.set_inner(VotePosition {
        authority: ctx.accounts.authority.key(),
        gauge: ctx.accounts.gauge.key(),
        vote_escrow: ctx.accounts.vote_escrow.key(),
        deposited: 0,
        self_bump: [ctx.bumps.vote_position],
    });
    Ok(())
}

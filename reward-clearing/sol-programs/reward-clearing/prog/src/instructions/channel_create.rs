use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    constants::{REWARD_CHANNEL_SEED, REWARD_STAGING_SEED},
    state::{channel::Channel, com::EpochTerms, organization::Organization, tenant::Tenant},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ChannelCreateArgs {
    pub default_epoch_terms: EpochTerms,
}

#[derive(Accounts)]
pub struct ChannelCreateAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(
        has_one = authority
    )]
    pub organization: Box<Account<'info, Organization>>,

    #[account(
        has_one = organization
    )]
    pub tenant: Box<Account<'info, Tenant>>,

    pub reward_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        space = 8 + Channel::INIT_SPACE,
        seeds = [
            REWARD_CHANNEL_SEED,
            tenant.key().as_ref(),
            reward_token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub channel: Box<Account<'info, Channel>>,

    #[account(
        init,
        payer = payer,
        seeds = [REWARD_STAGING_SEED, channel.key().as_ref()],
        bump,
        token::mint = reward_token_mint,
        token::authority = channel,
    )]
    pub reward_staging_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

pub fn channel_create_handler(
    ctx: Context<ChannelCreateAccounts>,
    args: ChannelCreateArgs,
) -> Result<()> {
    let channel = Channel {
        tenant: ctx.accounts.tenant.key(),
        reward_token_mint: ctx.accounts.reward_token_mint.key(),
        reward_staging_token_account: ctx.accounts.reward_staging_token_account.key(),
        default_epoch_terms: args.default_epoch_terms,
        disabled: false,
        self_bump: [ctx.bumps.channel],
    };

    ctx.accounts.channel.set_inner(channel);

    Ok(())
}

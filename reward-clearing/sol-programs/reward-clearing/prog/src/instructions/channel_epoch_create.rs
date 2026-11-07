use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::constants::{
    CHANNEL_EPOCH_CLEARING_CREDITS_ESCROW_SEED, CHANNEL_EPOCH_REWARD_POT_ESCROW_SEED,
    CHANNEL_EPOCH_SEED,
};
use crate::state::channel::Channel;
use crate::state::channel_epoch::ChannelEpoch;
use crate::state::epoch::Epoch;
use crate::state::tenant::Tenant;

/// Create a new epoch for a channel
#[derive(Accounts)]
pub struct ChannelEpochCreateAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        has_one = clearing_credits_mint
    )]
    pub tenant: Account<'info, Tenant>,

    #[account(
        has_one = tenant,
        has_one = reward_token_mint
    )]
    pub channel: Account<'info, Channel>,

    /// Validated that this is the current epoch for the tenant
    #[account(
        has_one = tenant,
        constraint = current_epoch.epoch_counter == tenant.current_epoch_index
    )]
    pub current_epoch: Account<'info, Epoch>,

    #[account(
        init,
        payer = payer,
        space = 8 + ChannelEpoch::INIT_SPACE,
        seeds = [CHANNEL_EPOCH_SEED, channel.key().as_ref(), current_epoch.key().as_ref()],
        bump,
    )]
    pub new_channel_epoch: Account<'info, ChannelEpoch>,

    /// CHECK: Validated by channel
    pub reward_token_mint: UncheckedAccount<'info>,

    /// CHECK: Validated by tenant
    pub clearing_credits_mint: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [CHANNEL_EPOCH_REWARD_POT_ESCROW_SEED, channel.key().as_ref(), current_epoch.key().as_ref()],
        bump,
        token::mint = reward_token_mint,
        token::authority = new_channel_epoch
    )]
    pub reward_pot_escrow_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        seeds = [CHANNEL_EPOCH_CLEARING_CREDITS_ESCROW_SEED, channel.key().as_ref(), current_epoch.key().as_ref()],
        bump,
        token::mint = clearing_credits_mint,
        token::authority = new_channel_epoch
    )]
    pub clearing_credits_escrow_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}

pub fn channel_epoch_create_handler(ctx: Context<ChannelEpochCreateAccounts>) -> Result<()> {
    let new_channel_epoch = ChannelEpoch {
        channel: ctx.accounts.channel.key(),
        epoch: ctx.accounts.current_epoch.key(),
        reward_pot_escrow_token_account: ctx.accounts.reward_pot_escrow_account.key(),
        clearing_credits_escrow_token_account: ctx.accounts.clearing_credits_escrow_account.key(),
        clearing_credits_balance: 0,
        reward_pot_balance: 0,
        weighted_shares_balance: 0,
        terms: ctx.accounts.channel.default_epoch_terms,
        self_bump: [ctx.bumps.new_channel_epoch],
    };

    ctx.accounts.new_channel_epoch.set_inner(new_channel_epoch);

    Ok(())
}

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use mayf_reward_clearing::{CreateChannelEpochArgs, create_channel_epoch};

use crate::clock::now_unix_seconds;
use crate::constants::{
    CHANNEL_EPOCH_CLEARING_CREDITS_ESCROW_SEED, CHANNEL_EPOCH_REWARD_POT_ESCROW_SEED,
    CHANNEL_EPOCH_SEED,
};
use crate::error::RewardClearingProgramError;
use crate::state::channel::Channel;
use crate::state::channel_epoch::ChannelEpoch;
use crate::state::channel_group::ChannelGroup;
use crate::state::epoch::Epoch;

/// Create a new epoch for a channel
#[derive(Accounts)]
pub struct ChannelEpochCreateAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(has_one = clearing_credits_mint)]
    pub channel_group: Account<'info, ChannelGroup>,

    #[account(mut, has_one = reward_token_mint)]
    pub channel: Account<'info, Channel>,

    pub current_epoch: Account<'info, Epoch>,

    #[account(
        init,
        payer = payer,
        space = 8 + ChannelEpoch::INIT_SPACE,
        seeds = [CHANNEL_EPOCH_SEED, channel.key().as_ref(), current_epoch.epoch_counter.to_le_bytes().as_ref()],
        bump,
    )]
    pub new_channel_epoch: Account<'info, ChannelEpoch>,

    /// CHECK: Validated by channel
    pub reward_token_mint: UncheckedAccount<'info>,

    /// CHECK: Validated by channel group
    pub clearing_credits_mint: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [CHANNEL_EPOCH_REWARD_POT_ESCROW_SEED, channel.key().as_ref(), current_epoch.epoch_counter.to_le_bytes().as_ref()],
        bump,
        token::mint = reward_token_mint,
        token::authority = new_channel_epoch
    )]
    pub reward_pot_escrow_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        seeds = [CHANNEL_EPOCH_CLEARING_CREDITS_ESCROW_SEED, channel.key().as_ref(), current_epoch.epoch_counter.to_le_bytes().as_ref()],
        bump,
        token::mint = clearing_credits_mint,
        token::authority = new_channel_epoch
    )]
    pub clearing_credits_escrow_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}

pub fn channel_epoch_create_handler(ctx: Context<ChannelEpochCreateAccounts>) -> Result<()> {
    let receipt = create_channel_epoch(CreateChannelEpochArgs {
        channel_group: ctx
            .accounts
            .channel_group
            .to_library(ctx.accounts.channel_group.key()),
        channel: ctx
            .accounts
            .channel
            .to_library(ctx.accounts.channel.key())
            .map_err(RewardClearingProgramError::from)?,
        active_epoch: (&*ctx.accounts.current_epoch).into(),
        now: now_unix_seconds()?,
    })
    .map_err(RewardClearingProgramError::from)?;

    let new_channel_epoch = ChannelEpoch::from_library(
        receipt.new_channel_epoch,
        ctx.accounts.reward_pot_escrow_account.key(),
        ctx.accounts.clearing_credits_escrow_account.key(),
        ctx.bumps.new_channel_epoch,
    );

    ctx.accounts.new_channel_epoch.set_inner(new_channel_epoch);
    ctx.accounts.channel.last_created_epoch_counter =
        receipt.channel_next.last_created_epoch_counter;

    Ok(())
}

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mayf_reward_clearing::{CreateChannelArgs, create_channel};

use crate::{
    clock::now_unix_seconds,
    constants::{
        CHANNEL_EPOCH_CLEARING_CREDITS_ESCROW_SEED, CHANNEL_EPOCH_REWARD_POT_ESCROW_SEED,
        CHANNEL_EPOCH_SEED, REWARD_CHANNEL_SEED, REWARD_STAGING_SEED,
    },
    error::RewardClearingProgramError,
    state::{
        channel::Channel,
        channel_epoch::ChannelEpoch,
        channel_group::ChannelGroup,
        com::{AccountId, EpochTerms},
        epoch::Epoch,
        organization::Organization,
    },
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
        has_one = organization,
        has_one = clearing_credits_mint
    )]
    pub channel_group: Box<Account<'info, ChannelGroup>>,

    pub current_epoch: Box<Account<'info, Epoch>>,

    pub reward_token_mint: Box<Account<'info, Mint>>,

    /// CHECK: Bound to channel_group; SPL Token validates escrow initialization.
    pub clearing_credits_mint: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + Channel::INIT_SPACE,
        seeds = [
            REWARD_CHANNEL_SEED,
            channel_group.key().as_ref(),
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

    #[account(
        init,
        payer = payer,
        space = 8 + ChannelEpoch::INIT_SPACE,
        seeds = [CHANNEL_EPOCH_SEED, channel.key().as_ref(), current_epoch.epoch_counter.to_le_bytes().as_ref()],
        bump,
    )]
    pub initial_channel_epoch: Box<Account<'info, ChannelEpoch>>,

    #[account(
        init,
        payer = payer,
        seeds = [CHANNEL_EPOCH_REWARD_POT_ESCROW_SEED, channel.key().as_ref(), current_epoch.epoch_counter.to_le_bytes().as_ref()],
        bump,
        token::mint = reward_token_mint,
        token::authority = initial_channel_epoch
    )]
    pub reward_pot_escrow_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer,
        seeds = [CHANNEL_EPOCH_CLEARING_CREDITS_ESCROW_SEED, channel.key().as_ref(), current_epoch.epoch_counter.to_le_bytes().as_ref()],
        bump,
        token::mint = clearing_credits_mint,
        token::authority = initial_channel_epoch
    )]
    pub clearing_credits_escrow_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

pub fn channel_create_handler(
    ctx: Context<ChannelCreateAccounts>,
    args: ChannelCreateArgs,
) -> Result<()> {
    let receipt = create_channel(CreateChannelArgs {
        channel_id: AccountId(ctx.accounts.channel.key()),
        reward_id: AccountId(ctx.accounts.reward_token_mint.key()),
        default_terms: args
            .default_epoch_terms
            .try_into()
            .map_err(RewardClearingProgramError::from)?,
        channel_group: ctx
            .accounts
            .channel_group
            .to_library(ctx.accounts.channel_group.key()),
        active_epoch: (&**ctx.accounts.current_epoch).into(),
        now: now_unix_seconds()?,
    })
    .map_err(RewardClearingProgramError::from)?;

    let channel = Channel {
        channel_group: receipt.new_channel.channel_group_id.0,
        reward_token_mint: receipt.new_channel.reward_id.0,
        reward_staging_token_account: ctx.accounts.reward_staging_token_account.key(),
        default_epoch_terms: args.default_epoch_terms,
        last_created_epoch_counter: receipt.new_channel.last_created_epoch_counter,
        disabled: receipt.new_channel.disabled,
        self_bump: [ctx.bumps.channel],
    };
    let channel_epoch = ChannelEpoch::from_library(
        receipt.new_channel_epoch,
        ctx.accounts.reward_pot_escrow_account.key(),
        ctx.accounts.clearing_credits_escrow_account.key(),
        ctx.bumps.initial_channel_epoch,
    );

    ctx.accounts.channel.set_inner(channel);
    ctx.accounts.initial_channel_epoch.set_inner(channel_epoch);

    Ok(())
}

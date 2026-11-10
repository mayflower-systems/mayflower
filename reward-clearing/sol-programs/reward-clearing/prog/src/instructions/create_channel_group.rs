use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mayf_reward_clearing::{
    CreateChannelGroupArgs as LibCreateChannelGroupArgs, create_channel_group,
};

use crate::constants::{CHANNEL_GROUP_SEED, CLEARING_TOKEN_ESCROW_SEED, EPOCH_SEED};
use crate::error::RewardClearingProgramError;
use crate::state::channel_group::ChannelGroup;
use crate::state::com::AccountId;
use crate::state::epoch::Epoch;
use crate::state::organization::Organization;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateChannelGroupArgs {
    pub master_start_time_unix_seconds: u64,
    pub epoch_duration_seconds: u32,
}

#[derive(Accounts)]
pub struct CreateChannelGroupAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority
    )]
    pub organization: Account<'info, Organization>,

    #[account(
        init,
        payer = payer,
        space = 8 + ChannelGroup::INIT_SPACE,
        seeds = [CHANNEL_GROUP_SEED, organization.key().as_ref(), organization.next_channel_group_index.to_le_bytes().as_ref()],
        bump
    )]
    pub channel_group: Account<'info, ChannelGroup>,

    #[account(
        init,
        payer = payer,
        space = 8 + Epoch::INIT_SPACE,
        seeds = [EPOCH_SEED, channel_group.key().as_ref(), channel_group.current_epoch_index.to_le_bytes().as_ref()],
        bump,
    )]
    pub initial_epoch: Account<'info, Epoch>,

    #[account(
        constraint = clearing_credits_mint.key() != anchor_spl::token::spl_token::native_mint::ID
            @ RewardClearingProgramError::NativeClearingMint
    )]
    pub clearing_credits_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        seeds = [CLEARING_TOKEN_ESCROW_SEED, channel_group.key().as_ref()],
        bump,
        token::mint = clearing_credits_mint,
        token::authority = channel_group
    )]
    pub clearing_credits_escrow: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn create_channel_group_handler(
    ctx: Context<CreateChannelGroupAccounts>,
    args: CreateChannelGroupArgs,
) -> Result<()> {
    let receipt = create_channel_group(LibCreateChannelGroupArgs {
        channel_group_id: AccountId(ctx.accounts.channel_group.key()),
        master_start_time_unix_seconds: args.master_start_time_unix_seconds,
        epoch_duration_seconds: args.epoch_duration_seconds.into(),
    })
    .map_err(RewardClearingProgramError::from)?;

    let next_channel_group_index = ctx
        .accounts
        .organization
        .next_channel_group_index
        .checked_add(1)
        .ok_or(RewardClearingProgramError::ChannelGroupIndexOverflow)?;
    let group = receipt.new_channel_group;
    let new_channel_group = ChannelGroup {
        organization: ctx.accounts.organization.key(),
        channel_group_index: ctx.accounts.organization.next_channel_group_index,
        master_start_time_unix_seconds: group.master_start_time_unix_seconds,
        epoch_duration_seconds: group
            .epoch_duration_seconds
            .try_into()
            .map_err(|_| RewardClearingProgramError::EpochDurationOutOfRange)?,
        clearing_credits_mint: ctx.accounts.clearing_credits_mint.key(),
        self_bump: [ctx.bumps.channel_group],
        current_epoch_index: group.current_epoch_index,
    };
    let initial_epoch = receipt.initial_epoch.try_into()?;

    ctx.accounts.channel_group.set_inner(new_channel_group);
    ctx.accounts.initial_epoch.set_inner(initial_epoch);
    ctx.accounts.organization.next_channel_group_index = next_channel_group_index;

    Ok(())
}

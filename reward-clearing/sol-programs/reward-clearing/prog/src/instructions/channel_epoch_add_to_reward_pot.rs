use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Transfer, transfer};
use mayf_reward_clearing::{AddRewardArgs, add_reward};

use crate::clock::now_unix_seconds;
use crate::error::RewardClearingProgramError;
use crate::state::channel::Channel;
use crate::state::channel_epoch::ChannelEpoch;
use crate::state::channel_group::ChannelGroup;
use crate::state::epoch::Epoch;

#[derive(Accounts)]
pub struct ChannelEpochAddToRewardPotAccounts<'info> {
    pub transfer_authority: Signer<'info>,

    pub channel_group: Account<'info, ChannelGroup>,

    pub channel: Account<'info, Channel>,

    pub epoch: Account<'info, Epoch>,

    #[account(mut, has_one = reward_pot_escrow_token_account)]
    pub channel_epoch: Account<'info, ChannelEpoch>,

    /// CHECK: Validated by channel_epoch
    #[account(mut)]
    pub reward_pot_escrow_token_account: UncheckedAccount<'info>,

    /// CHECK: Validated by token program
    #[account(mut)]
    pub reward_token_src: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn channel_epoch_add_to_reward_pot_handler(
    ctx: Context<ChannelEpochAddToRewardPotAccounts>,
    amount: u64,
) -> Result<()> {
    let receipt = add_reward(AddRewardArgs {
        channel_group: ctx
            .accounts
            .channel_group
            .to_library(ctx.accounts.channel_group.key()),
        channel: ctx
            .accounts
            .channel
            .to_library(ctx.accounts.channel.key())
            .map_err(RewardClearingProgramError::from)?,
        channel_epoch: (&*ctx.accounts.channel_epoch)
            .try_into()
            .map_err(RewardClearingProgramError::from)?,
        epoch: (&*ctx.accounts.epoch).into(),
        now: now_unix_seconds()?,
        amount,
    })
    .map_err(RewardClearingProgramError::from)?;

    let stored = &ctx.accounts.channel_epoch;
    let funded = ChannelEpoch::from_library(
        receipt.channel_epoch_next,
        stored.reward_pot_escrow_token_account,
        stored.clearing_credits_escrow_token_account,
        stored.self_bump[0],
    );

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.reward_token_src.to_account_info(),
                to: ctx
                    .accounts
                    .reward_pot_escrow_token_account
                    .to_account_info(),
                authority: ctx.accounts.transfer_authority.to_account_info(),
            },
        ),
        amount,
    )?;

    ctx.accounts.channel_epoch.set_inner(funded);

    Ok(())
}

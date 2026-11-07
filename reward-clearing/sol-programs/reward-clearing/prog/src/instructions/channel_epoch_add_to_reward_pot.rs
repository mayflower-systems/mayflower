use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Transfer, transfer};

use crate::clock::now_unix_seconds;
use crate::state::channel_epoch::ChannelEpoch;
use crate::state::epoch::Epoch;

#[derive(Accounts)]
pub struct ChannelEpochAddToRewardPotAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub transfer_authority: Signer<'info>,

    #[account(
        constraint = epoch.is_active(now_unix_seconds())
    )]
    pub epoch: Account<'info, Epoch>,

    #[account(
        mut,
        has_one = reward_pot_escrow_token_account,
        has_one = epoch
    )]
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

    ctx.accounts.channel_epoch.reward_pot_balance = ctx
        .accounts
        .channel_epoch
        .reward_pot_balance
        .checked_add(amount)
        .unwrap();

    Ok(())
}

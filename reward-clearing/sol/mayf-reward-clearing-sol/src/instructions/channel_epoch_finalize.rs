use anchor_lang::prelude::*;
use anchor_spl::token::{Burn, Token, burn};

use crate::clock::now_unix_seconds;
use crate::constants::CHANNEL_EPOCH_FINAL_SNAPSHOT_SEED;
use crate::state::channel::Channel;
use crate::state::channel_epoch::ChannelEpoch;
use crate::state::channel_epoch_final::ChannelEpochFinal;
use crate::state::epoch::Epoch;
use crate::state::tenant::Tenant;

// There is one ChannelEpochFinal account per completed ChannelEpoch
#[derive(Accounts)]
pub struct ChannelEpochFinalizeAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        has_one = clearing_credits_mint
    )]
    pub tenant: Account<'info, Tenant>,

    #[account(
        has_one = tenant,
    )]
    pub channel: Account<'info, Channel>,

    #[account(
        has_one = tenant,
        constraint = completed_epoch.is_finished(now_unix_seconds())
    )]
    pub completed_epoch: Account<'info, Epoch>,

    #[account(
        // link to channel
        has_one = channel,
        // link to epoch
        constraint = completed_channel_epoch.epoch == completed_epoch.key(),
        // link to clearing credits escrow for burning
        has_one = clearing_credits_escrow_token_account
    )]
    pub completed_channel_epoch: Account<'info, ChannelEpoch>,

    #[account(
        init,
        payer = payer,
        space = 8 + ChannelEpochFinal::INIT_SPACE,
        seeds = [CHANNEL_EPOCH_FINAL_SNAPSHOT_SEED, completed_channel_epoch.key().as_ref()],
        bump,
    )]
    pub channel_epoch_final_snapshot: Account<'info, ChannelEpochFinal>,

    /// CHECK: Validated by tenant
    pub clearing_credits_mint: UncheckedAccount<'info>,

    /// CHECK: Validated by completed_channel_epoch
    pub clearing_credits_escrow_token_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}

impl<'info> ChannelEpochFinalizeAccounts<'info> {
    fn burn_accounts(&self) -> Burn<'info> {
        Burn {
            mint: self.clearing_credits_mint.to_account_info(),
            from: self.clearing_credits_escrow_token_account.to_account_info(),
            authority: self.completed_channel_epoch.to_account_info(),
        }
    }
}

pub fn channel_epoch_finalize_handler(ctx: Context<ChannelEpochFinalizeAccounts>) -> Result<()> {
    let channel_epoch_final = ChannelEpochFinal {
        channel_epoch: ctx.accounts.completed_channel_epoch.key(),
        unclaimed_weighted_shares: ctx.accounts.completed_channel_epoch.weighted_shares_balance,
    };

    ctx.accounts
        .channel_epoch_final_snapshot
        .set_inner(channel_epoch_final);

    burn_clearing_credits(ctx)?;

    Ok(())
}

fn burn_clearing_credits(ctx: Context<ChannelEpochFinalizeAccounts>) -> Result<()> {
    burn(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            ctx.accounts.burn_accounts(),
        )
        .with_signer(&[&ctx.accounts.completed_channel_epoch.signer_seeds()]),
        ctx.accounts
            .completed_channel_epoch
            .clearing_credits_balance,
    )
}

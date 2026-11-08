use anchor_lang::prelude::*;
use anchor_spl::token::{Burn, Token, burn};
use mayf_reward_clearing::{FinalizeChannelEpochArgs, finalize_channel_epoch};

use crate::clock::now_unix_seconds;
use crate::error::RewardClearingProgramError;
use crate::state::channel::Channel;
use crate::state::channel_epoch::ChannelEpoch;
use crate::state::epoch::Epoch;

#[derive(Accounts)]
pub struct ChannelEpochFinalizeAccounts<'info> {
    pub channel: Account<'info, Channel>,

    pub completed_epoch: Account<'info, Epoch>,

    #[account(mut, has_one = clearing_credits_escrow_token_account)]
    pub completed_channel_epoch: Account<'info, ChannelEpoch>,

    /// CHECK: Validated by SPL Token when burning remaining credits.
    #[account(mut)]
    pub clearing_credits_mint: UncheckedAccount<'info>,

    /// CHECK: Bound to completed_channel_epoch; SPL Token validates the burn.
    #[account(mut)]
    pub clearing_credits_escrow_token_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

impl ChannelEpochFinalizeAccounts<'_> {
    fn burn_clearing_credits(&self) -> Result<()> {
        let amount = self.completed_channel_epoch.clearing_credits_balance;
        if amount == 0 {
            return Ok(());
        }

        let epoch_counter = self.completed_channel_epoch.epoch_counter.to_le_bytes();
        let signer_seeds = self.completed_channel_epoch.signer_seeds(&epoch_counter);
        burn(
            CpiContext::new(
                self.token_program.key(),
                Burn {
                    mint: self.clearing_credits_mint.to_account_info(),
                    from: self.clearing_credits_escrow_token_account.to_account_info(),
                    authority: self.completed_channel_epoch.to_account_info(),
                },
            )
            .with_signer(&[&signer_seeds]),
            amount,
        )
    }
}

pub fn channel_epoch_finalize_handler(ctx: Context<ChannelEpochFinalizeAccounts>) -> Result<()> {
    let receipt = finalize_channel_epoch(FinalizeChannelEpochArgs {
        channel: ctx
            .accounts
            .channel
            .to_library(ctx.accounts.channel.key())
            .map_err(RewardClearingProgramError::from)?,
        channel_epoch: (&*ctx.accounts.completed_channel_epoch)
            .try_into()
            .map_err(RewardClearingProgramError::from)?,
        epoch: (&*ctx.accounts.completed_epoch).into(),
        now: now_unix_seconds()?,
    })
    .map_err(RewardClearingProgramError::from)?;

    let stored = &ctx.accounts.completed_channel_epoch;
    let finalized = ChannelEpoch::from_library(
        receipt.channel_epoch_next,
        stored.reward_pot_escrow_token_account,
        stored.clearing_credits_escrow_token_account,
        stored.self_bump[0],
    );

    ctx.accounts.burn_clearing_credits()?;
    ctx.accounts.completed_channel_epoch.set_inner(finalized);
    Ok(())
}

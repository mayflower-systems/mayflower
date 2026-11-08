use anchor_lang::prelude::*;
use anchor_spl::token::{Burn, Token, Transfer, burn, transfer};
use mayf_reward_clearing::{WithdrawCreditsArgs, withdraw_credits};

use crate::clock::now_unix_seconds;
use crate::error::RewardClearingProgramError;
use crate::state::{
    bidder_position::BidderPosition, channel::Channel, channel_epoch::ChannelEpoch,
    channel_group::ChannelGroup, epoch::Epoch,
};

#[derive(Accounts)]
pub struct BidderPositionWithdrawAccounts<'info> {
    pub authority: Signer<'info>,

    pub channel_group: Account<'info, ChannelGroup>,

    pub channel: Account<'info, Channel>,

    pub epoch: Account<'info, Epoch>,

    #[account(mut, has_one = clearing_credits_escrow_token_account)]
    pub channel_epoch: Account<'info, ChannelEpoch>,

    #[account(mut, has_one = authority)]
    pub bidder_position: Account<'info, BidderPosition>,

    /// CHECK: Validated by SPL Token when burning forfeited credits.
    #[account(mut)]
    pub clearing_credits_mint: UncheckedAccount<'info>,

    /// CHECK: Bound to channel_epoch; SPL Token validates settlement.
    #[account(mut)]
    pub clearing_credits_escrow_token_account: UncheckedAccount<'info>,

    /// CHECK: Chosen by the bidder; SPL Token validates the refund transfer.
    #[account(mut)]
    pub clearing_credits_destination: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

impl BidderPositionWithdrawAccounts<'_> {
    fn settle_withdrawal(&self, credits_returned: u64, credits_forfeited: u64) -> Result<()> {
        let epoch_counter = self.channel_epoch.epoch_counter.to_le_bytes();
        let signer_seeds = self.channel_epoch.signer_seeds(&epoch_counter);

        if credits_forfeited > 0 {
            burn(
                CpiContext::new(
                    self.token_program.key(),
                    Burn {
                        mint: self.clearing_credits_mint.to_account_info(),
                        from: self.clearing_credits_escrow_token_account.to_account_info(),
                        authority: self.channel_epoch.to_account_info(),
                    },
                )
                .with_signer(&[&signer_seeds]),
                credits_forfeited,
            )?;
        }

        if credits_returned > 0 {
            transfer(
                CpiContext::new(
                    self.token_program.key(),
                    Transfer {
                        from: self.clearing_credits_escrow_token_account.to_account_info(),
                        to: self.clearing_credits_destination.to_account_info(),
                        authority: self.channel_epoch.to_account_info(),
                    },
                )
                .with_signer(&[&signer_seeds]),
                credits_returned,
            )?;
        }
        Ok(())
    }
}

pub fn bidder_position_withdraw_handler(
    ctx: Context<BidderPositionWithdrawAccounts>,
    amount: u64,
) -> Result<()> {
    let receipt = withdraw_credits(WithdrawCreditsArgs {
        channel_group: ctx
            .accounts
            .channel_group
            .to_library(ctx.accounts.channel_group.key()),
        bidder_position: ctx
            .accounts
            .bidder_position
            .to_library(ctx.accounts.bidder_position.key()),
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
        credits_to_withdraw: amount,
    })
    .map_err(RewardClearingProgramError::from)?;

    let stored_position = &ctx.accounts.bidder_position;
    let position_next = BidderPosition::from_library(
        receipt.bidder_position_next,
        stored_position.authority,
        stored_position.self_bump[0],
    );
    let stored_epoch = &ctx.accounts.channel_epoch;
    let epoch_next = ChannelEpoch::from_library(
        receipt.channel_epoch_next,
        stored_epoch.reward_pot_escrow_token_account,
        stored_epoch.clearing_credits_escrow_token_account,
        stored_epoch.self_bump[0],
    );

    ctx.accounts
        .settle_withdrawal(receipt.credits_returned, receipt.credits_forfeited)?;
    ctx.accounts.bidder_position.set_inner(position_next);
    ctx.accounts.channel_epoch.set_inner(epoch_next);
    Ok(())
}

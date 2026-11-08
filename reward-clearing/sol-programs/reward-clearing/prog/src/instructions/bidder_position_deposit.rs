use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{Token, Transfer, transfer};
use mayf_reward_clearing::{DepositCreditsArgs, deposit_credits};

use crate::clock::now_unix_seconds;
use crate::constants::MAX_LOT_COUNT;
use crate::error::RewardClearingProgramError;
use crate::state::{
    bidder_position::BidderPosition, channel::Channel, channel_epoch::ChannelEpoch,
    channel_group::ChannelGroup, epoch::Epoch,
};

#[derive(Accounts)]
pub struct BidderPositionDepositAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    pub channel_group: Account<'info, ChannelGroup>,

    pub channel: Account<'info, Channel>,

    pub epoch: Account<'info, Epoch>,

    #[account(mut, has_one = clearing_credits_escrow_token_account)]
    pub channel_epoch: Account<'info, ChannelEpoch>,

    #[account(mut, has_one = authority)]
    pub bidder_position: Account<'info, BidderPosition>,

    /// CHECK: Bound to channel_epoch; SPL Token validates the deposit transfer.
    #[account(mut)]
    pub clearing_credits_escrow_token_account: UncheckedAccount<'info>,

    /// CHECK: Validated by SPL Token during the deposit transfer.
    #[account(mut)]
    pub clearing_credits_source: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl BidderPositionDepositAccounts<'_> {
    fn grow_position(&self, space: usize) -> Result<()> {
        let account = self.bidder_position.to_account_info();
        if space <= account.data_len() {
            return Ok(());
        }

        let rent_shortfall = Rent::get()?
            .minimum_balance(space)
            .saturating_sub(account.lamports());
        if rent_shortfall > 0 {
            system_program::transfer(
                CpiContext::new(
                    self.system_program.key(),
                    system_program::Transfer {
                        from: self.payer.to_account_info(),
                        to: account.clone(),
                    },
                ),
                rent_shortfall,
            )?;
        }
        account.resize(space)?;
        Ok(())
    }
}

pub fn bidder_position_deposit_handler(
    ctx: Context<BidderPositionDepositAccounts>,
    amount: u64,
) -> Result<()> {
    let receipt = deposit_credits(DepositCreditsArgs {
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
        credits_to_deposit: amount,
        max_lot_count: MAX_LOT_COUNT,
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

    ctx.accounts.grow_position(position_next.space())?;
    transfer(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.clearing_credits_source.to_account_info(),
                to: ctx
                    .accounts
                    .clearing_credits_escrow_token_account
                    .to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        amount,
    )?;

    ctx.accounts.bidder_position.set_inner(position_next);
    ctx.accounts.channel_epoch.set_inner(epoch_next);
    Ok(())
}

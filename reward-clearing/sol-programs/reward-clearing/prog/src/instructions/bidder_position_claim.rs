use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Transfer, transfer};
use mayf_reward_clearing::{ClaimArgs, claim};

use crate::error::RewardClearingProgramError;
use crate::state::{bidder_position::BidderPosition, channel_epoch::ChannelEpoch};

#[derive(Accounts)]
pub struct BidderPositionClaimAccounts<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, has_one = reward_pot_escrow_token_account)]
    pub channel_epoch: Account<'info, ChannelEpoch>,

    #[account(mut, has_one = authority, close = authority)]
    pub bidder_position: Account<'info, BidderPosition>,

    /// CHECK: Bound to channel_epoch; SPL Token validates the payout transfer.
    #[account(mut)]
    pub reward_pot_escrow_token_account: UncheckedAccount<'info>,

    /// CHECK: Must differ from the escrow; SPL Token validates the payout transfer.
    #[account(
        mut,
        constraint = reward_destination.key() != reward_pot_escrow_token_account.key()
            @ RewardClearingProgramError::ClaimDestinationIsEscrow
    )]
    pub reward_destination: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

impl BidderPositionClaimAccounts<'_> {
    fn pay_reward(&self, amount: u64) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }

        let epoch_counter = self.channel_epoch.epoch_counter.to_le_bytes();
        let signer_seeds = self.channel_epoch.signer_seeds(&epoch_counter);
        transfer(
            CpiContext::new(
                self.token_program.key(),
                Transfer {
                    from: self.reward_pot_escrow_token_account.to_account_info(),
                    to: self.reward_destination.to_account_info(),
                    authority: self.channel_epoch.to_account_info(),
                },
            )
            .with_signer(&[&signer_seeds]),
            amount,
        )
    }
}

pub fn bidder_position_claim_handler(ctx: Context<BidderPositionClaimAccounts>) -> Result<()> {
    let receipt = claim(ClaimArgs {
        bidder_position: ctx
            .accounts
            .bidder_position
            .to_library(ctx.accounts.bidder_position.key()),
        channel_epoch: (&*ctx.accounts.channel_epoch)
            .try_into()
            .map_err(RewardClearingProgramError::from)?,
    })
    .map_err(RewardClearingProgramError::from)?;

    let stored_epoch = &ctx.accounts.channel_epoch;
    let epoch_next = ChannelEpoch::from_library(
        receipt.channel_epoch_next,
        stored_epoch.reward_pot_escrow_token_account,
        stored_epoch.clearing_credits_escrow_token_account,
        stored_epoch.self_bump[0],
    );

    ctx.accounts.pay_reward(receipt.reward_earned)?;
    ctx.accounts.channel_epoch.set_inner(epoch_next);
    Ok(())
}

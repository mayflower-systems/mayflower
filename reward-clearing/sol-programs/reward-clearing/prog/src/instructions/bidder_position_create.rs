use anchor_lang::prelude::*;
use mayf_reward_clearing::{CreateBidderPositionArgs, create_bidder_position};

use crate::clock::now_unix_seconds;
use crate::constants::BIDDER_POSITION_SEED;
use crate::error::RewardClearingProgramError;
use crate::state::{
    bidder_position::BidderPosition, channel::Channel, channel_epoch::ChannelEpoch,
    channel_group::ChannelGroup, com::AccountId, epoch::Epoch,
};

#[derive(Accounts)]
pub struct BidderPositionCreateAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    pub channel_group: Account<'info, ChannelGroup>,

    pub channel: Account<'info, Channel>,

    pub epoch: Account<'info, Epoch>,

    pub channel_epoch: Account<'info, ChannelEpoch>,

    #[account(
        init,
        payer = payer,
        space = BidderPosition::BASE_SPACE,
        seeds = [
            BIDDER_POSITION_SEED,
            channel_epoch.channel.as_ref(),
            channel_epoch.epoch_counter.to_le_bytes().as_ref(),
            authority.key().as_ref()
        ],
        bump
    )]
    pub bidder_position: Account<'info, BidderPosition>,

    pub system_program: Program<'info, System>,
}

pub fn bidder_position_create_handler(ctx: Context<BidderPositionCreateAccounts>) -> Result<()> {
    let receipt = create_bidder_position(CreateBidderPositionArgs {
        position_id: AccountId(ctx.accounts.bidder_position.key()),
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
    })
    .map_err(RewardClearingProgramError::from)?;
    ctx.accounts
        .bidder_position
        .set_inner(BidderPosition::from_library(
            receipt.bidder_position,
            ctx.accounts.authority.key(),
            ctx.bumps.bidder_position,
        ));
    Ok(())
}

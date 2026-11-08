use anchor_lang::prelude::*;
use mayf_reward_clearing::{AdvanceEpochArgs, advance_epoch};

use crate::clock::now_unix_seconds;
use crate::constants::EPOCH_SEED;
use crate::error::RewardClearingProgramError;
use crate::state::channel_group::ChannelGroup;
use crate::state::epoch::Epoch;

#[derive(Accounts)]
pub struct EpochCreateAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub channel_group: Account<'info, ChannelGroup>,

    pub current_epoch: Account<'info, Epoch>,

    #[account(
        init,
        payer = payer,
        space = 8 + Epoch::INIT_SPACE,
        seeds = [
            EPOCH_SEED,
            channel_group.key().as_ref(),
            channel_group.to_library(channel_group.key()).next_epoch_index()
                .map_err(RewardClearingProgramError::from)?.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub new_epoch: Account<'info, Epoch>,

    pub system_program: Program<'info, System>,
}

pub fn epoch_create_handler(ctx: Context<EpochCreateAccounts>) -> Result<()> {
    let receipt = advance_epoch(AdvanceEpochArgs {
        now: now_unix_seconds()?,
        channel_group: ctx
            .accounts
            .channel_group
            .to_library(ctx.accounts.channel_group.key()),
        current_epoch: (&*ctx.accounts.current_epoch).into(),
    })
    .map_err(RewardClearingProgramError::from)?;

    ctx.accounts
        .new_epoch
        .set_inner(receipt.new_epoch.try_into()?);
    ctx.accounts.channel_group.current_epoch_index = receipt.channel_group_next.current_epoch_index;

    Ok(())
}

use anchor_lang::prelude::*;
use mayf_fixed_drivers::U64F64;
use mayf_reward_clearing::EpochTerms as LibEpochTerms;

use crate::error::RewardClearingProgramError;

use crate::state::{
    channel::Channel, channel_group::ChannelGroup, com::EpochTerms, organization::Organization,
};

#[derive(Accounts)]
pub struct ChannelSetDefaultEpochTermsAccounts<'info> {
    pub authority: Signer<'info>,

    #[account(
        has_one = authority
    )]
    pub organization: Box<Account<'info, Organization>>,

    #[account(
        has_one = organization
    )]
    pub channel_group: Box<Account<'info, ChannelGroup>>,

    #[account(
        mut,
        has_one = channel_group
    )]
    pub channel: Account<'info, Channel>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ChannelSetDefaultEpochTermsArgs {
    pub default_epoch_terms: EpochTerms,
}

pub fn channel_set_default_epoch_terms_handler(
    ctx: Context<ChannelSetDefaultEpochTermsAccounts>,
    args: ChannelSetDefaultEpochTermsArgs,
) -> Result<()> {
    LibEpochTerms::<U64F64>::try_from(args.default_epoch_terms)
        .map_err(RewardClearingProgramError::from)?;

    ctx.accounts.channel.default_epoch_terms = args.default_epoch_terms;
    Ok(())
}

use anchor_lang::prelude::*;

use crate::state::{channel::Channel, com::EpochTerms, organization::Organization, tenant::Tenant};

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
    pub tenant: Box<Account<'info, Tenant>>,

    #[account(
        mut,
        has_one = tenant
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
    ctx.accounts.channel.default_epoch_terms = args.default_epoch_terms;
    Ok(())
}

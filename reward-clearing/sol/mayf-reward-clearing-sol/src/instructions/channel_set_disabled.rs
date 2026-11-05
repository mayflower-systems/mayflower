use anchor_lang::prelude::*;

use crate::state::{channel::Channel, organization::Organization, tenant::Tenant};

#[derive(Accounts)]
pub struct ChannelSetDisabledAccounts<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(has_one = authority)]
    pub organization: Box<Account<'info, Organization>>,

    #[account(has_one = organization)]
    pub tenant: Box<Account<'info, Tenant>>,

    #[account(mut, has_one = tenant)]
    pub channel: Account<'info, Channel>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ChannelSetDisabledArgs {
    pub disabled: bool,
}

pub fn channel_set_disabled_handler(
    ctx: Context<ChannelSetDisabledAccounts>,
    args: ChannelSetDisabledArgs,
) -> Result<()> {
    ctx.accounts.channel.disabled = args.disabled;
    Ok(())
}

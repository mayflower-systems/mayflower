use anchor_lang::prelude::*;

use crate::state::{channel::Channel, channel_group::ChannelGroup, organization::Organization};

#[derive(Accounts)]
pub struct ChannelSetDisabledAccounts<'info> {
    pub authority: Signer<'info>,

    #[account(has_one = authority)]
    pub organization: Box<Account<'info, Organization>>,

    #[account(has_one = organization)]
    pub channel_group: Box<Account<'info, ChannelGroup>>,

    #[account(
        mut,
        has_one = channel_group
    )]
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

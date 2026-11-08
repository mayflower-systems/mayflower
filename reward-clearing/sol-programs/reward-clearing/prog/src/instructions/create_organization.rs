use anchor_lang::prelude::*;

use crate::{constants::ORGANIZATION_SEED, state::organization::Organization};

#[derive(Accounts)]
pub struct CreateOrganization<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + Organization::INIT_SPACE,
        seeds = [ORGANIZATION_SEED, authority.key().as_ref()],
        bump
    )]
    pub organization: Account<'info, Organization>,

    pub system_program: Program<'info, System>,
}

pub fn create_organization_handler(ctx: Context<CreateOrganization>) -> Result<()> {
    ctx.accounts.organization.authority = ctx.accounts.authority.key();
    ctx.accounts.organization.next_channel_group_index = 0;
    Ok(())
}

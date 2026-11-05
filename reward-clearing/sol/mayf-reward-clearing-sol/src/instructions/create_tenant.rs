use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::{CLEARING_TOKEN_ESCROW_SEED, EPOCH_SEED, TENANT_SEED};
use crate::state::epoch::Epoch;
use crate::state::organization::Organization;
use crate::state::tenant::Tenant;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateTenantArgs {
    pub master_start_time_unix_seconds: u64,
    pub epoch_duration_seconds: u32,
}

#[derive(Accounts)]
pub struct CreateTenantAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority
    )]
    pub organization: Account<'info, Organization>,

    #[account(
        init,
        payer = payer,
        space = 8 + Tenant::INIT_SPACE,
        seeds = [TENANT_SEED, organization.key().as_ref(), organization.next_tenant_index.to_le_bytes().as_ref()],
        bump
    )]
    pub tenant: Account<'info, Tenant>,

    #[account(
        init,
        payer = payer,
        space = 8 + Epoch::INIT_SPACE,
        seeds = [EPOCH_SEED, tenant.key().as_ref(), tenant.current_epoch_index.to_le_bytes().as_ref()],
        bump,
    )]
    pub initial_epoch: Account<'info, Epoch>,

    pub clearing_credits_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        seeds = [CLEARING_TOKEN_ESCROW_SEED, tenant.key().as_ref()],
        bump,
        token::mint = clearing_credits_mint,
        token::authority = tenant
    )]
    pub clearing_credits_escrow: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn create_tenant_handler(
    ctx: Context<CreateTenantAccounts>,
    args: CreateTenantArgs,
) -> Result<()> {
    let new_tenant = Tenant {
        organization: ctx.accounts.organization.key(),
        tenant_index: ctx.accounts.organization.next_tenant_index,
        master_start_time_unix_seconds: args.master_start_time_unix_seconds,
        epoch_duration_seconds: args.epoch_duration_seconds,
        clearing_credits_mint: ctx.accounts.clearing_credits_mint.key(),
        self_bump: [ctx.bumps.tenant],
        current_epoch_index: 0,
    };

    let initial_epoch = Epoch {
        tenant: ctx.accounts.tenant.key(),
        epoch_counter: 0,
        start_time_unix_seconds: args.master_start_time_unix_seconds,
        duration_seconds: args.epoch_duration_seconds,
    };

    ctx.accounts.tenant.set_inner(new_tenant);
    ctx.accounts.initial_epoch.set_inner(initial_epoch);

    advance_tenant_index(&mut ctx.accounts.organization);

    Ok(())
}

fn advance_tenant_index(organization: &mut Organization) {
    let cur_index = organization.next_tenant_index;
    let next_index = cur_index.checked_add(1).unwrap();
    organization.next_tenant_index = next_index;
}

use anchor_lang::prelude::*;

use crate::clock::now_unix_seconds;
use crate::constants::EPOCH_SEED;
use crate::state::epoch::Epoch;
use crate::state::tenant::Tenant;

// Can only create a new epoch if:
// - The current epoch is what the tenant expects to be the current epoch
// - The current epoch is finished

#[derive(Accounts)]
pub struct EpochCreateAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub tenant: Account<'info, Tenant>,

    #[account(
        has_one = tenant,

        // Ensure the current epoch is the current epoch for the tenant
        constraint = current_epoch.epoch_counter == tenant.current_epoch_index,

        // Ensure the current epoch is finished
        constraint = current_epoch.is_finished(now_unix_seconds())
    )]
    pub current_epoch: Account<'info, Epoch>,

    #[account(
        init,
        payer = payer,
        space = 8 + Epoch::INIT_SPACE,
        seeds = [EPOCH_SEED, tenant.key().as_ref(), tenant.safe_next_epoch_index().to_le_bytes().as_ref()],
        bump,
    )]
    pub new_epoch: Account<'info, Epoch>,

    pub system_program: Program<'info, System>,
}

pub fn epoch_create_handler(ctx: Context<EpochCreateAccounts>) -> Result<()> {
    let next_epoch_index = ctx.accounts.tenant.safe_next_epoch_index();

    let new_epoch = Epoch {
        tenant: ctx.accounts.tenant.key(),
        epoch_counter: next_epoch_index,
        start_time_unix_seconds: ctx.accounts.current_epoch.end_time_unix_seconds(),
        duration_seconds: ctx.accounts.tenant.epoch_duration_seconds,
    };

    ctx.accounts.new_epoch.set_inner(new_epoch);

    ctx.accounts.tenant.current_epoch_index = next_epoch_index;

    Ok(())
}

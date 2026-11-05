#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

mod clock;
mod constants;
mod instructions;
mod state;

use instructions::*;

declare_id!("RCgecPmsA1YfbwBZ9FkPmGUkKqnEGwvMrpnzUhUthxd");

#[program]
pub mod mayf_reward_clearing_sol {
    use super::*;

    pub fn create_organization(ctx: Context<CreateOrganization>) -> Result<()> {
        create_organization_handler(ctx)
    }

    pub fn create_tenant(ctx: Context<CreateTenantAccounts>, args: CreateTenantArgs) -> Result<()> {
        create_tenant_handler(ctx, args)
    }

    pub fn channel_create(
        ctx: Context<ChannelCreateAccounts>,
        args: ChannelCreateArgs,
    ) -> Result<()> {
        channel_create_handler(ctx, args)
    }

    pub fn channel_set_default_epoch_terms(
        ctx: Context<ChannelSetDefaultEpochTermsAccounts>,
        args: ChannelSetDefaultEpochTermsArgs,
    ) -> Result<()> {
        channel_set_default_epoch_terms_handler(ctx, args)
    }

    pub fn channel_set_disabled(
        ctx: Context<ChannelSetDisabledAccounts>,
        args: ChannelSetDisabledArgs,
    ) -> Result<()> {
        channel_set_disabled_handler(ctx, args)
    }

    pub fn channel_epoch_create(ctx: Context<ChannelEpochCreateAccounts>) -> Result<()> {
        channel_epoch_create_handler(ctx)
    }

    pub fn channel_epoch_finalize(ctx: Context<ChannelEpochFinalizeAccounts>) -> Result<()> {
        channel_epoch_finalize_handler(ctx)
    }

    pub fn channel_epoch_add_to_reward_pot(
        ctx: Context<ChannelEpochAddToRewardPotAccounts>,
        amount: u64,
    ) -> Result<()> {
        channel_epoch_add_to_reward_pot_handler(ctx, amount)
    }

    pub fn epoch_create(ctx: Context<EpochCreateAccounts>) -> Result<()> {
        epoch_create_handler(ctx)
    }
}

#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

mod clock;
mod constants;
mod error;
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

    pub fn create_channel_group(
        ctx: Context<CreateChannelGroupAccounts>,
        args: CreateChannelGroupArgs,
    ) -> Result<()> {
        create_channel_group_handler(ctx, args)
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

    pub fn bidder_position_create(ctx: Context<BidderPositionCreateAccounts>) -> Result<()> {
        bidder_position_create_handler(ctx)
    }

    pub fn bidder_position_deposit(
        ctx: Context<BidderPositionDepositAccounts>,
        amount: u64,
    ) -> Result<()> {
        bidder_position_deposit_handler(ctx, amount)
    }

    pub fn bidder_position_withdraw(
        ctx: Context<BidderPositionWithdrawAccounts>,
        amount: u64,
    ) -> Result<()> {
        bidder_position_withdraw_handler(ctx, amount)
    }

    pub fn bidder_position_claim(ctx: Context<BidderPositionClaimAccounts>) -> Result<()> {
        bidder_position_claim_handler(ctx)
    }
}

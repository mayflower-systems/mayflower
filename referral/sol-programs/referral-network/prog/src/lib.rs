#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

mod constants;
mod error;
mod instructions;
mod state;

use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod referral_network {
    use super::*;

    pub fn create_network(
        ctx: Context<CreateNetworkAccounts>,
        args: CreateNetworkArgs,
    ) -> Result<()> {
        create_network_handler(ctx, args)
    }

    pub fn register_lane(ctx: Context<RegisterLaneAccounts>, args: RegisterLaneArgs) -> Result<()> {
        register_lane_handler(ctx, args)
    }

    pub fn set_balance(ctx: Context<SetBalanceAccounts>, args: SetBalanceArgs) -> Result<()> {
        set_balance_handler(ctx, args)
    }

    pub fn join_at_top_level(
        ctx: Context<JoinAtTopLevelAccounts>,
        args: JoinAtTopLevelArgs,
    ) -> Result<()> {
        join_at_top_level_handler(ctx, args)
    }

    pub fn join_under_parent(
        ctx: Context<JoinUnderParentAccounts>,
        args: JoinUnderParentArgs,
    ) -> Result<()> {
        join_under_parent_handler(ctx, args)
    }

    pub fn set_offer(ctx: Context<SetOfferAccounts>, args: SetOfferArgs) -> Result<()> {
        set_offer_handler(ctx, args)
    }

    pub fn set_sharing(ctx: Context<SetSharingAccounts>, args: SetSharingArgs) -> Result<()> {
        set_sharing_handler(ctx, args)
    }

    pub fn set_credit_stake(
        ctx: Context<SetCreditStakeAccounts>,
        args: SetCreditStakeArgs,
    ) -> Result<()> {
        set_credit_stake_handler(ctx, args)
    }

    pub fn collect_credits(ctx: Context<CollectCreditsAccounts>) -> Result<()> {
        collect_credits_handler(ctx)
    }

    pub fn settle_node_in_lane(ctx: Context<SettleNodeInLaneAccounts>) -> Result<()> {
        settle_node_in_lane_handler(ctx)
    }

    pub fn settle_family(ctx: Context<SettleFamilyAccounts>) -> Result<()> {
        settle_family_handler(ctx)
    }
}

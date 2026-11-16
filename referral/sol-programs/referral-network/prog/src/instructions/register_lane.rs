use anchor_lang::prelude::*;

use crate::{
    constants::LANE_SEED,
    error::ReferralNetworkProgramError,
    state::{lane::Lane, network::Network},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterLaneArgs {
    pub asset: Pubkey,
    pub earnings_source: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: RegisterLaneArgs)]
pub struct RegisterLaneAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority
    )]
    pub network: Account<'info, Network>,

    #[account(
        init,
        payer = payer,
        space = 8 + Lane::INIT_SPACE,
        seeds = [LANE_SEED, network.key().as_ref(), args.asset.as_ref()],
        bump
    )]
    pub lane: Account<'info, Lane>,

    pub system_program: Program<'info, System>,
}

pub fn register_lane_handler(
    _ctx: Context<RegisterLaneAccounts>,
    _args: RegisterLaneArgs,
) -> Result<()> {
    // TODO: Authenticate the earnings source and read its current baseline;
    // call mayf_referral::program::register_lane and store its complete receipt.
    err!(ReferralNetworkProgramError::InstructionNotImplemented)
}

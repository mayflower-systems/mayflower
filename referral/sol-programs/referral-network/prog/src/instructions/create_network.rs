use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use mayf_referral::program::{CreateProtocolArgs, create_protocol};

use crate::{
    constants::NETWORK_SEED,
    error::ReferralNetworkProgramError,
    state::{com::NetworkTerms, network::Network},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateNetworkArgs {
    pub terms: NetworkTerms,
}

#[derive(Accounts)]
pub struct CreateNetworkAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + Network::INIT_SPACE,
        seeds = [NETWORK_SEED, authority.key().as_ref()],
        bump
    )]
    pub network: Account<'info, Network>,

    #[account(
        constraint = credits_mint.key() != anchor_spl::token::spl_token::native_mint::ID
            @ ReferralNetworkProgramError::NativeCreditsMint
    )]
    pub credits_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
}

pub fn create_network_handler(
    ctx: Context<CreateNetworkAccounts>,
    args: CreateNetworkArgs,
) -> Result<()> {
    let receipt = create_protocol(CreateProtocolArgs {
        terms: args.terms.try_into()?,
    });

    ctx.accounts.network.set_inner(Network {
        authority: ctx.accounts.authority.key(),
        credits_mint: ctx.accounts.credits_mint.key(),
        terms: receipt.new_protocol.terms.into(),
        lane_count: receipt.new_protocol.lane_count,
        self_bump: [ctx.bumps.network],
    });
    Ok(())
}

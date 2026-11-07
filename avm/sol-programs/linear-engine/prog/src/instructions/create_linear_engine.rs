use anchor_lang::prelude::*;

use crate::{error::EngineError, state::LinearCharacteristicParam};

#[derive(Accounts)]
pub struct CreateLinearEngineAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = LinearCharacteristicParam::DISCRIMINATOR.len() + LinearCharacteristicParam::INIT_SPACE,
    )]
    pub linear_characteristic_param: Account<'info, LinearCharacteristicParam>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateLinearEngineArgs {
    pub slope_q64x64: u128,
}

pub fn create_linear_engine_handler(
    ctx: Context<CreateLinearEngineAccounts>,
    args: CreateLinearEngineArgs,
) -> Result<()> {
    require!(args.slope_q64x64 > 0, EngineError::InvalidSlope);
    ctx.accounts.linear_characteristic_param.slope_q64x64 = args.slope_q64x64;
    Ok(())
}

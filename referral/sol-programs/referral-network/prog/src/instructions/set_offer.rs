use anchor_lang::prelude::*;

use crate::{
    error::ReferralNetworkProgramError,
    state::{com::Offer, network::Network, node::Node},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetOfferArgs {
    pub offer: Offer,
}

#[derive(Accounts)]
pub struct SetOfferAccounts<'info> {
    pub owner: Signer<'info>,

    pub network: Account<'info, Network>,

    #[account(
        mut,
        has_one = network,
        has_one = owner
    )]
    pub node: Box<Account<'info, Node>>,
}

pub fn set_offer_handler(_ctx: Context<SetOfferAccounts>, _args: SetOfferArgs) -> Result<()> {
    // TODO: Convert the node to kernel state, call set_offer, and persist the
    // returned node without losing any accounting field. No settlement is needed.
    err!(ReferralNetworkProgramError::InstructionNotImplemented)
}

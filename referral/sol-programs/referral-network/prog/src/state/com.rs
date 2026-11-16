use anchor_lang::prelude::*;
use mayf_referral::{MicroBasisPoints, NetworkTerms as LibNetworkTerms};

use crate::error::ReferralNetworkProgramError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, InitSpace, PartialEq, Eq)]
pub struct NetworkTerms {
    pub max_depth: u8,
    /// One hundred percent is 1,000,000 micro-basis points.
    pub override_rate_mbps: u32,
}

impl TryFrom<NetworkTerms> for LibNetworkTerms {
    type Error = ReferralNetworkProgramError;

    fn try_from(terms: NetworkTerms) -> core::result::Result<Self, Self::Error> {
        let rate = MicroBasisPoints::new(terms.override_rate_mbps)
            .ok_or(ReferralNetworkProgramError::InvalidRate)?;
        Self::new(terms.max_depth, rate).map_err(ReferralNetworkProgramError::from)
    }
}

impl From<LibNetworkTerms> for NetworkTerms {
    fn from(terms: LibNetworkTerms) -> Self {
        Self {
            max_depth: terms.max_depth(),
            override_rate_mbps: terms.override_rate().get(),
        }
    }
}

/// Ordinary admission pricing in already-issued credit base units.
#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, InitSpace, PartialEq, Eq,
)]
pub struct Offer {
    pub current_fee: u64,
    pub increment: u64,
}

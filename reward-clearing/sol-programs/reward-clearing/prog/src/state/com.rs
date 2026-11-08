use anchor_lang::prelude::*;
use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::DirectedRounding;
use mayf_reward_clearing::{
    EpochTerms as LibEpochTerms, RewardClearingError, models::Identifiable,
};

/// An account address used as an identity by the pure state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccountId(pub Pubkey);

impl Identifiable for AccountId {}

#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, InitSpace, PartialEq, Eq,
)]
pub struct EpochTerms {
    /// Ramp bonus slope in micro-basis points
    pub ramp_bonus_slope_mbps: u32,

    /// Exit penalty exponent
    pub exit_penalty_exponent: u8,
}

impl TryFrom<EpochTerms> for LibEpochTerms<U64F64> {
    type Error = RewardClearingError;

    fn try_from(terms: EpochTerms) -> mayf_reward_clearing::Result<Self> {
        let slope = mayf_micro_basis_points::raw_to_fixed(
            terms.ramp_bonus_slope_mbps,
            DirectedRounding::Floor,
        )
        .map_err(|_| RewardClearingError::InvalidRampBonusSlope)?;
        Self::new(slope, terms.exit_penalty_exponent.into())
    }
}

/// An epoch stores the library's exact numeric snapshot, without converting it
/// back to micro-basis points and introducing another rounding step.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, InitSpace, PartialEq, Eq)]
pub struct StoredEpochTerms {
    pub ramp_bonus_slope_q64f64: u128,
    pub exit_penalty_exponent: u32,
}

impl From<LibEpochTerms<U64F64>> for StoredEpochTerms {
    fn from(terms: LibEpochTerms<U64F64>) -> Self {
        Self {
            ramp_bonus_slope_q64f64: terms.ramp_bonus_slope().get().to_scaled_u128(),
            exit_penalty_exponent: terms.exit_penalty_exponent(),
        }
    }
}

impl TryFrom<StoredEpochTerms> for LibEpochTerms<U64F64> {
    type Error = RewardClearingError;

    fn try_from(terms: StoredEpochTerms) -> mayf_reward_clearing::Result<Self> {
        Self::new(
            U64F64::from_scaled_u128(terms.ramp_bonus_slope_q64f64),
            terms.exit_penalty_exponent,
        )
    }
}

use anchor_lang::prelude::*;

#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, InitSpace, PartialEq, Eq,
)]
pub struct EpochTerms {
    /// Ramp bonus slope in micro-basis points
    pub ramp_bonus_slope_mbps: u32,

    /// Exit penalty exponent
    pub exit_penalty_exponent: u8,
}

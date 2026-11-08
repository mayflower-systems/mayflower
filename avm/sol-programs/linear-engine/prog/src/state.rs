use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LinearCharacteristicParam {
    /// Positive, immutable slope of `f(x) = slope * x`, on the `2^-64` grid.
    pub slope_q64x64: u128,
}

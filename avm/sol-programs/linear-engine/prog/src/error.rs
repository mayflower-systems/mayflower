use anchor_lang::prelude::*;

#[error_code]
pub enum EngineError {
    #[msg("The characteristic slope must be positive")]
    InvalidSlope,
    #[msg("The span responses would exceed Solana's return-data limit")]
    BatchTooLarge,
    #[msg("The span or its result exceeds the supported fixed-point range")]
    ArithmeticFailure,
}

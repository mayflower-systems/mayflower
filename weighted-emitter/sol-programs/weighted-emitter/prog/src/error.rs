use anchor_lang::prelude::*;
use mayf_weighted_emitter::WeightedEmitterError;

#[error_code]
pub enum WeightedEmitterProgramError {
    #[msg("The clock moved backwards")]
    TimeWentBack,
    #[msg("The gauge belongs to a different emitter")]
    GaugeEmitterMismatch,
    #[msg("The supplied accounting state is inconsistent")]
    InvalidState,
    #[msg("The accounting number has insufficient precision")]
    InsufficientPrecision,
    #[msg("An accounting amount overflowed")]
    ArithmeticOverflow,
    #[msg("The clock timestamp is negative")]
    InvalidTimestamp,
    #[msg("The native mint cannot be an emission mint")]
    NativeEmissionMint,
    #[msg("The emission mint authority must be the emitter's mint authority PDA")]
    InvalidMintAuthority,
    #[msg("A vote deposit or withdrawal must be positive")]
    ZeroAmount,
    #[msg("The withdrawal exceeds this position's recorded deposit")]
    InsufficientDeposit,
    #[msg("The vote escrow does not back the recorded deposit")]
    EscrowUnderfunded,
    #[msg("The deposit source cannot be its own vote escrow")]
    DepositSourceIsEscrow,
    #[msg("The withdrawal destination cannot be its own vote escrow")]
    WithdrawalDestinationIsEscrow,
}

impl From<WeightedEmitterError> for WeightedEmitterProgramError {
    fn from(error: WeightedEmitterError) -> Self {
        match error {
            WeightedEmitterError::TimeWentBack => Self::TimeWentBack,
            WeightedEmitterError::GaugeEmitterMismatch => Self::GaugeEmitterMismatch,
            WeightedEmitterError::InvalidState => Self::InvalidState,
            WeightedEmitterError::InsufficientPrecision => Self::InsufficientPrecision,
            WeightedEmitterError::ArithmeticOverflow => Self::ArithmeticOverflow,
        }
    }
}

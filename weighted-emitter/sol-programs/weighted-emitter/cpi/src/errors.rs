//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]

// errors ------------------------------------------------------------

/// Errors the program returns as `InstructionError::Custom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MayfWeightedEmitterError {
    /// The clock moved backwards
    TimeWentBack = 6000,
    /// The gauge belongs to a different emitter
    GaugeEmitterMismatch = 6001,
    /// The supplied accounting state is inconsistent
    InvalidState = 6002,
    /// The accounting number has insufficient precision
    InsufficientPrecision = 6003,
    /// An accounting amount overflowed
    ArithmeticOverflow = 6004,
    /// The clock timestamp is negative
    InvalidTimestamp = 6005,
    /// The native mint cannot be an emission mint
    NativeEmissionMint = 6006,
    /// The emission mint authority must be the emitter's mint authority PDA
    InvalidMintAuthority = 6007,
    /// A vote deposit or withdrawal must be positive
    ZeroAmount = 6008,
    /// The withdrawal exceeds this position's recorded deposit
    InsufficientDeposit = 6009,
    /// The vote escrow does not back the recorded deposit
    EscrowUnderfunded = 6010,
    /// The deposit source cannot be its own vote escrow
    DepositSourceIsEscrow = 6011,
    /// The withdrawal destination cannot be its own vote escrow
    WithdrawalDestinationIsEscrow = 6012,
}

impl MayfWeightedEmitterError {
    /// Every variant, in IDL order.
    pub const ALL: &'static [Self] = &[
        Self::TimeWentBack,
        Self::GaugeEmitterMismatch,
        Self::InvalidState,
        Self::InsufficientPrecision,
        Self::ArithmeticOverflow,
        Self::InvalidTimestamp,
        Self::NativeEmissionMint,
        Self::InvalidMintAuthority,
        Self::ZeroAmount,
        Self::InsufficientDeposit,
        Self::EscrowUnderfunded,
        Self::DepositSourceIsEscrow,
        Self::WithdrawalDestinationIsEscrow,
    ];

    /// Recover the variant a custom error code refers to.
    pub fn from_code(code: u32) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|error| *error as u32 == code)
    }

    /// The variant name, as the IDL spells it.
    pub fn name(self) -> &'static str {
        match self {
            Self::TimeWentBack => "TimeWentBack",
            Self::GaugeEmitterMismatch => "GaugeEmitterMismatch",
            Self::InvalidState => "InvalidState",
            Self::InsufficientPrecision => "InsufficientPrecision",
            Self::ArithmeticOverflow => "ArithmeticOverflow",
            Self::InvalidTimestamp => "InvalidTimestamp",
            Self::NativeEmissionMint => "NativeEmissionMint",
            Self::InvalidMintAuthority => "InvalidMintAuthority",
            Self::ZeroAmount => "ZeroAmount",
            Self::InsufficientDeposit => "InsufficientDeposit",
            Self::EscrowUnderfunded => "EscrowUnderfunded",
            Self::DepositSourceIsEscrow => "DepositSourceIsEscrow",
            Self::WithdrawalDestinationIsEscrow => "WithdrawalDestinationIsEscrow",
        }
    }

    /// The message the program declares for this error.
    pub fn message(self) -> &'static str {
        match self {
            Self::TimeWentBack => "The clock moved backwards",
            Self::GaugeEmitterMismatch => "The gauge belongs to a different emitter",
            Self::InvalidState => "The supplied accounting state is inconsistent",
            Self::InsufficientPrecision => "The accounting number has insufficient precision",
            Self::ArithmeticOverflow => "An accounting amount overflowed",
            Self::InvalidTimestamp => "The clock timestamp is negative",
            Self::NativeEmissionMint => "The native mint cannot be an emission mint",
            Self::InvalidMintAuthority => {
                "The emission mint authority must be the emitter's mint authority PDA"
            }
            Self::ZeroAmount => "A vote deposit or withdrawal must be positive",
            Self::InsufficientDeposit => "The withdrawal exceeds this position's recorded deposit",
            Self::EscrowUnderfunded => "The vote escrow does not back the recorded deposit",
            Self::DepositSourceIsEscrow => "The deposit source cannot be its own vote escrow",
            Self::WithdrawalDestinationIsEscrow => {
                "The withdrawal destination cannot be its own vote escrow"
            }
        }
    }
}

impl core::fmt::Display for MayfWeightedEmitterError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ({}): {}", self.name(), *self as u32, self.message())
    }
}

impl std::error::Error for MayfWeightedEmitterError {}

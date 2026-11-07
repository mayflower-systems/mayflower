//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]

// errors ------------------------------------------------------------

/// Errors the program returns as `InstructionError::Custom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MayfAvmEngineLinearError {
    /// The characteristic slope must be positive
    InvalidSlope = 6000,
    /// The span responses would exceed Solana's return-data limit
    BatchTooLarge = 6001,
    /// The span or its result exceeds the supported fixed-point range
    ArithmeticFailure = 6002,
}

impl MayfAvmEngineLinearError {
    /// Every variant, in IDL order.
    pub const ALL: &'static [Self] = &[
        Self::InvalidSlope,
        Self::BatchTooLarge,
        Self::ArithmeticFailure,
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
            Self::InvalidSlope => "InvalidSlope",
            Self::BatchTooLarge => "BatchTooLarge",
            Self::ArithmeticFailure => "ArithmeticFailure",
        }
    }

    /// The message the program declares for this error.
    pub fn message(self) -> &'static str {
        match self {
            Self::InvalidSlope => "The characteristic slope must be positive",
            Self::BatchTooLarge => "The span responses would exceed Solana's return-data limit",
            Self::ArithmeticFailure => {
                "The span or its result exceeds the supported fixed-point range"
            }
        }
    }
}

impl core::fmt::Display for MayfAvmEngineLinearError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ({}): {}", self.name(), *self as u32, self.message())
    }
}

impl std::error::Error for MayfAvmEngineLinearError {}

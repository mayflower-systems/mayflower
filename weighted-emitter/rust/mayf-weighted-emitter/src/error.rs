use mayf_fixed_number::MathError;

/// Failures leave the host's original records unchanged: actions consume drafts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeightedEmitterError {
    TimeWentBack,
    GaugeEmitterMismatch,
    /// The supplied records contain inconsistent checkpoints, weights or totals.
    InvalidState,
    /// The number type cannot retain a quantum as small as 2^-64 base units.
    InsufficientPrecision,
    ArithmeticOverflow,
}

pub type Result<T> = core::result::Result<T, WeightedEmitterError>;

impl From<MathError> for WeightedEmitterError {
    fn from(error: MathError) -> Self {
        match error {
            MathError::Overflow | MathError::UnsupportedRange => Self::ArithmeticOverflow,
            MathError::DivisionByZero
            | MathError::Inexact
            | MathError::NegativeToUnsigned
            | MathError::Domain => Self::InvalidState,
        }
    }
}

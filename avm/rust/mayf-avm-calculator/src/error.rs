use mayf_monotone::MonotoneCalculatorError;

use crate::characteristic::CharacteristicEngineError;

/// Failures in schedule geometry, quote acceptance, or characteristic arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    RampWidthExceedsEnd,
    RampEndBeyondProgress,
    InsufficientCirculation,
    InsufficientFloorTail,
    FloorNotRaised,
    RampMovedBackward,
    MainCurveChanged,
    AreaChangeOutsideTolerance,
    InvalidCharacteristicBatch,
    CharacteristicOutOfDomain,
    InvalidInterval,
    NegativeOutput,
    ArithmeticFailure,
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<CharacteristicEngineError> for Error {
    fn from(error: CharacteristicEngineError) -> Self {
        match error {
            CharacteristicEngineError::InvalidBatch => Self::InvalidCharacteristicBatch,
            CharacteristicEngineError::Calculation(error) => error.into(),
        }
    }
}

impl From<MonotoneCalculatorError> for Error {
    fn from(error: MonotoneCalculatorError) -> Self {
        match error {
            MonotoneCalculatorError::InputOutOfDomain => Self::CharacteristicOutOfDomain,
            MonotoneCalculatorError::InvalidInterval => Self::InvalidInterval,
            MonotoneCalculatorError::NegativeOutput => Self::NegativeOutput,
            MonotoneCalculatorError::ArithmeticFailure => Self::ArithmeticFailure,
        }
    }
}

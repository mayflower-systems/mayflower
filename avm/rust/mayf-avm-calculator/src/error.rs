use mayf_characteristic::CharacteristicError;

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
    NegativeOutput,
    ArithmeticFailure,
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<CharacteristicEngineError> for Error {
    fn from(error: CharacteristicEngineError) -> Self {
        match error {
            CharacteristicEngineError::InvalidBatch => Self::InvalidCharacteristicBatch,
            CharacteristicEngineError::Calculation(CharacteristicError::InputOutOfDomain) => {
                Self::CharacteristicOutOfDomain
            }
            CharacteristicEngineError::Calculation(CharacteristicError::ArithmeticFailure) => {
                Self::ArithmeticFailure
            }
        }
    }
}

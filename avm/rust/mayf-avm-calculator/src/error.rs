use mayf_fixed_number::MathResult;
use mayf_monotone::MonotoneCalculatorError;

/// Failures in schedule geometry, quote acceptance, or characteristic arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    RampWidthExceedsEnd,
    RampEndBeyondProgress,
    InsufficientCirculation,
    InsufficientFloorTail,
    FloorNotRaised,
    RampMovedBackward,
    RampNarrowed,
    MainCurveChanged,
    BackingIncreased,
    ReserveSurplusExceeded,
    CharacteristicOutOfDomain,
    InvalidInterval,
    NegativeOutput,
    ArithmeticFailure,
}

pub type Result<T> = core::result::Result<T, Error>;

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

pub(crate) fn arithmetic<T>(result: MathResult<T>) -> Result<T> {
    result.map_err(|_| Error::ArithmeticFailure)
}

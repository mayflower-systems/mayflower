use mayf_fixed_number::MathResult;
use mayf_monotone::MonotoneCalculatorError;

/// A structural or arithmetic failure while constructing or evaluating a schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleError {
    /// Ramp width must not extend left of coordinate zero.
    RampWidthExceedsEnd,
    /// Main evaluations must remain in the reference's forward domain.
    MainReferenceAfterRampEnd,
    /// A schedule beginning on main at zero must be anchored at its floor.
    InvalidOriginReference,
    /// A horizontal translation cannot move the visible ramp start below zero.
    NegativeRampStart,
    /// A component or transformed input is outside the characteristic's domain.
    CharacteristicOutOfDomain,
    /// A component rejected an integration interval.
    InvalidInterval,
    /// A component could not certify a wholly nonnegative result.
    NegativeOutput,
    /// Checked arithmetic could not represent a required value.
    ArithmeticFailure,
}

pub type ScheduleResult<T> = Result<T, ScheduleError>;

impl From<MonotoneCalculatorError> for ScheduleError {
    fn from(error: MonotoneCalculatorError) -> Self {
        match error {
            MonotoneCalculatorError::InputOutOfDomain => Self::CharacteristicOutOfDomain,
            MonotoneCalculatorError::InvalidInterval => Self::InvalidInterval,
            MonotoneCalculatorError::NegativeOutput => Self::NegativeOutput,
            MonotoneCalculatorError::ArithmeticFailure => Self::ArithmeticFailure,
        }
    }
}

pub(crate) fn arithmetic<T>(result: MathResult<T>) -> ScheduleResult<T> {
    result.map_err(|_| ScheduleError::ArithmeticFailure)
}

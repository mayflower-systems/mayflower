use mayf_fixed_number::MathError;

/// Rejected inputs and checked arithmetic failures. Failed actions return no state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidDepth,
    OverrideRateTooLargeForDepth,
    ParentAtMaxDepth,
    BrokenLineage,
    StrayLaneNode,
    MissingLaneNode,
    InvalidLane,
    TooManyLanes,
    IncompleteSources,
    WrongParent,
    JoinFeeExceeded,
    SeatAlreadyActivated,
    TimeWentBack,
    IndexWentBack,
    InvalidEmission,
    /// An amount or arithmetic result cannot be represented.
    NumberOutOfRange,
    /// Invalid numeric state, such as a negative earning amount or zero divisor.
    InvalidAccountingState,
    /// The selected number driver cannot perform a required exact operation.
    UnsupportedArithmetic,
    GridTooCoarse,
}

impl From<MathError> for Error {
    fn from(error: MathError) -> Self {
        match error {
            MathError::Overflow | MathError::NegativeToUnsigned => Self::NumberOutOfRange,
            MathError::DivisionByZero | MathError::Domain => Self::InvalidAccountingState,
            MathError::Inexact | MathError::UnsupportedRange => Self::UnsupportedArithmetic,
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

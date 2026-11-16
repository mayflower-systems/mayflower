use anchor_lang::prelude::*;
use mayf_referral::Error as ReferralError;

/// Referral domain failures and Solana host errors.
#[error_code]
pub enum ReferralNetworkProgramError {
    #[msg("This instruction is not implemented yet")]
    InstructionNotImplemented,
    #[msg("A rate must be between zero and 1,000,000 micro-basis points")]
    InvalidRate,
    #[msg("The native mint cannot be used for credits")]
    NativeCreditsMint,
    #[msg("The node depth exceeds the network's maximum depth")]
    InvalidDepth,
    #[msg("The maximum depth times the override rate must be less than one")]
    OverrideRateTooLargeForDepth,
    #[msg("A node at maximum depth cannot accept children")]
    ParentAtMaxDepth,
    #[msg("The supplied ancestry is incomplete or inconsistent")]
    BrokenLineage,
    #[msg("A lane position does not belong to the supplied members")]
    StrayLaneNode,
    #[msg("A previously materialized lane position is missing")]
    MissingLaneNode,
    #[msg("The lane is not registered in this network")]
    InvalidLane,
    #[msg("The network has reached its lane limit")]
    TooManyLanes,
    #[msg("The supplied lanes do not exactly cover the family's sources")]
    IncompleteSources,
    #[msg("The child does not belong to the supplied parent")]
    WrongParent,
    #[msg("The parent's join fee exceeds the accepted maximum")]
    JoinFeeExceeded,
    #[msg("The auction award has already been activated")]
    SeatAlreadyActivated,
    #[msg("A reward index is less than a previously consumed checkpoint")]
    IndexWentBack,
    #[msg("Cumulative earnings are negative or less than the consumed total")]
    InvalidEarnings,
    #[msg("An amount or arithmetic result cannot be represented")]
    NumberOutOfRange,
    #[msg("The accounting state contains an invalid numeric value")]
    InvalidAccountingState,
    #[msg("The number driver cannot perform a required exact operation")]
    UnsupportedArithmetic,
    #[msg("The accounting number has insufficient fractional precision")]
    GridTooCoarse,
}

impl From<ReferralError> for ReferralNetworkProgramError {
    fn from(error: ReferralError) -> Self {
        match error {
            ReferralError::InvalidDepth => Self::InvalidDepth,
            ReferralError::OverrideRateTooLargeForDepth => Self::OverrideRateTooLargeForDepth,
            ReferralError::ParentAtMaxDepth => Self::ParentAtMaxDepth,
            ReferralError::BrokenLineage => Self::BrokenLineage,
            ReferralError::StrayLaneNode => Self::StrayLaneNode,
            ReferralError::MissingLaneNode => Self::MissingLaneNode,
            ReferralError::InvalidLane => Self::InvalidLane,
            ReferralError::TooManyLanes => Self::TooManyLanes,
            ReferralError::IncompleteSources => Self::IncompleteSources,
            ReferralError::WrongParent => Self::WrongParent,
            ReferralError::JoinFeeExceeded => Self::JoinFeeExceeded,
            ReferralError::SeatAlreadyActivated => Self::SeatAlreadyActivated,
            ReferralError::IndexWentBack => Self::IndexWentBack,
            ReferralError::InvalidEarnings => Self::InvalidEarnings,
            ReferralError::NumberOutOfRange => Self::NumberOutOfRange,
            ReferralError::InvalidAccountingState => Self::InvalidAccountingState,
            ReferralError::UnsupportedArithmetic => Self::UnsupportedArithmetic,
            ReferralError::GridTooCoarse => Self::GridTooCoarse,
        }
    }
}

//! Semantic error seam for reward-clearing actions.
//!
//! The table defines the algorithm's errors and messages. Consumers can use
//! [`with_reward_clearing_errors`] to generate exhaustive mappings into their
//! own error types. Application failures and error codes belong to consumers.

/// Expand `$callback!` with the complete, ordered error table.
///
/// Each row is `Variant = "message"`.
#[macro_export]
macro_rules! with_reward_clearing_errors {
    ($callback:path $(, $($prefix:tt)*)?) => {
        $callback! {
            $($($prefix)*)?
            InvalidEpochDuration                 = "the epoch duration must be greater than zero",
            InvalidRampBonusSlope                = "the ramp bonus slope must be between zero and two",
            InvalidExitPenaltyExponent           = "the exit penalty exponent must be between one and four",
            EpochEndOverflow                     = "the epoch end timestamp overflowed",
            TimestampBeforeEpoch                 = "the timestamp is before the epoch start",
            EpochEnded                           = "the epoch has ended",
            EpochNotEnded                        = "the epoch has not ended",
            ScheduleNotStarted                   = "the supplied time is before the immutable schedule's master start",
            EpochCounterOverflow                 = "the wall-clock epoch counter exceeds its u32 range",
            EpochScheduleMismatch                = "the epoch is not aligned to its immutable schedule",
            EpochDurationMismatch                = "the epoch terms duration differs from its schedule window",
            FinalizedAccountingExceedsSettlement = "a finalized epoch remainder exceeds its immutable settlement totals",
            EpochClaimsOutstanding               = "the finalized epoch still has weighted shares eligible to claim",
            InvalidEpochAccounting               = "active-epoch principal and shares are inconsistent",
            InvalidPositionAccounting            = "position totals and the supplied lot are inconsistent",
            InvalidLotAccounting                 = "a deposit lot violates its local accounting bounds",
            LotWeightOutOfOrder                  = "the new deposit lot would violate LIFO weight ordering",
            PositionAlreadyClaimed               = "the position has already claimed",
            WithdrawalExceedsLotPrincipal        = "the withdrawal exceeds the current lot",
            WithdrawalTooSmall                   = "the withdrawal amount must be positive",
            PositionExceedsEpoch                 = "the position exceeds the epoch accounting",
            LotExceedsPosition                   = "the deposit lot exceeds the position accounting",
            EmptyLot                             = "an empty deposit lot cannot remain on the stack",
            DepositTooSmall                      = "the deposit is too small to issue a weighted share",
            ArithmeticFailure                    = "checked reward-clearing arithmetic failed",
        }
    };
}

macro_rules! define_core_errors {
    ($($variant:ident = $message:literal,)*) => {
        /// A failure while calculating a reward-clearing state transition.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(u32)]
        pub enum RewardClearingError {
            $(
                #[doc = $message]
                $variant,
            )*
        }

        impl RewardClearingError {
            /// The human-readable description carried by the error table.
            pub const fn message(self) -> &'static str {
                match self {
                    $(Self::$variant => $message,)*
                }
            }
        }
    };
}

with_reward_clearing_errors!(define_core_errors);

impl RewardClearingError {
    /// Position in this crate's error table.
    pub const fn ordinal(self) -> u32 {
        self as u32
    }
}

/// Result type used by this crate.
pub type Result<T> = core::result::Result<T, RewardClearingError>;

impl core::fmt::Display for RewardClearingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.message())
    }
}

impl core::error::Error for RewardClearingError {}

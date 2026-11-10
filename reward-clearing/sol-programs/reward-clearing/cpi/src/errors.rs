//! Generated from an Anchor IDL. Do not edit.

#![allow(clippy::too_many_arguments)]

// errors ------------------------------------------------------------

/// Errors the program returns as `InstructionError::Custom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MayfRewardClearingError {
    /// The epoch duration is zero
    InvalidEpochDuration = 6000,
    /// The ramp bonus slope is outside [0, 2]
    InvalidRampBonusSlope = 6001,
    /// The exit penalty exponent is outside [1, 4]
    InvalidExitPenaltyExponent = 6002,
    /// The time is before the epoch starts
    EpochNotStarted = 6003,
    /// The time is at or after the epoch's end
    EpochEnded = 6004,
    /// The time is before the epoch's end
    EpochNotEnded = 6005,
    /// The epoch's start time plus its duration overflows
    EpochEndTimeOverflow = 6006,
    /// The next epoch counter overflows
    EpochCounterOverflow = 6007,
    /// The epoch belongs to a different channel group
    EpochChannelGroupMismatch = 6008,
    /// The channel group's current epoch counter differs from the epoch's
    ChannelGroupCounterMismatch = 6009,
    /// The channel and the epoch belong to different channel groups
    ChannelGroupMismatch = 6010,
    /// The channel already opened a channel epoch for this epoch or a later one
    ChannelEpochAlreadyCreated = 6011,
    /// The channel epoch belongs to a different channel
    ChannelEpochChannelMismatch = 6012,
    /// The channel epoch's epoch counter differs from the epoch's
    ChannelEpochCounterMismatch = 6013,
    /// The bidder position belongs to a different channel epoch
    BidderPositionChannelEpochMismatch = 6014,
    /// A lot belongs to a different bidder position
    BidderLotPositionMismatch = 6015,
    /// The channel epoch has already been finalized
    ChannelEpochAlreadyFinalized = 6016,
    /// The channel epoch has not been finalized, so it cannot pay claims yet
    ChannelEpochNotFinalized = 6017,
    /// The bidder position has claimed, so it is frozen
    BidderPositionClaimed = 6018,
    /// The deposit is too small to issue any weighted shares
    DepositTooSmall = 6019,
    /// The withdrawal is zero credits
    WithdrawalTooSmall = 6020,
    /// The withdrawal is more than the position's credits
    WithdrawalExceedsPositionCredits = 6021,
    /// The entry weight does not fit the number type
    EntryWeightOverflow = 6022,
    /// The exit penalty does not fit the number type
    ExitPenaltyOverflow = 6023,
    /// A new lot's shares do not fit the number type
    LotSharesOverflow = 6024,
    /// The shares removed from a lot do not fit the number type
    LotSharesRemovedOverflow = 6025,
    /// A withdrawal takes more credits than the lot holds
    LotCreditsUnderflow = 6026,
    /// A withdrawal removes more shares than the lot holds
    LotSharesUnderflow = 6027,
    /// The shares removed across all lots overflow
    WithdrawnSharesOverflow = 6028,
    /// The credits returned after the exit penalty do not fit the number type
    CreditsReturnedOverflow = 6029,
    /// The credits returned exceed the credits withdrawn
    CreditsForfeitedUnderflow = 6030,
    /// The reward pot balance overflows
    RewardPotOverflow = 6031,
    /// A claim pays more reward than the reward pot holds
    RewardPotUnderflow = 6032,
    /// The clearing credits balance overflows
    CreditBalanceOverflow = 6033,
    /// A withdrawal takes more credits than the clearing credits balance holds
    CreditBalanceUnderflow = 6034,
    /// The weighted shares balance overflows
    WeightedSharesOverflow = 6035,
    /// A withdrawal or claim removes more shares than the weighted shares balance holds
    WeightedSharesUnderflow = 6036,
    /// The position has no shares to claim
    NoSharesToClaim = 6037,
    /// The position's shares overflow when summed across its lots
    ClaimedSharesOverflow = 6038,
    /// The claim's reward does not fit the number type
    ClaimRewardOverflow = 6039,
    /// The channel is disabled
    ChannelDisabled = 6040,
    /// The clock timestamp is negative
    InvalidTimestamp = 6041,
    /// The epoch duration exceeds the stored u32 seconds range
    EpochDurationOutOfRange = 6042,
    /// The bidder position has reached its lot limit
    LotLimitReached = 6043,
    /// The epoch duration must be at least one hour (3600 seconds)
    EpochDurationTooShort = 6044,
    /// The organization's next channel group index overflows
    ChannelGroupIndexOverflow = 6045,
    /// The native mint cannot be used for clearing credits
    NativeClearingMint = 6046,
    /// The withdrawal destination cannot be the clearing credits escrow
    WithdrawalDestinationIsEscrow = 6047,
    /// The claim destination cannot be the reward pot escrow
    ClaimDestinationIsEscrow = 6048,
}

impl MayfRewardClearingError {
    /// Every variant, in IDL order.
    pub const ALL: &'static [Self] = &[
        Self::InvalidEpochDuration,
        Self::InvalidRampBonusSlope,
        Self::InvalidExitPenaltyExponent,
        Self::EpochNotStarted,
        Self::EpochEnded,
        Self::EpochNotEnded,
        Self::EpochEndTimeOverflow,
        Self::EpochCounterOverflow,
        Self::EpochChannelGroupMismatch,
        Self::ChannelGroupCounterMismatch,
        Self::ChannelGroupMismatch,
        Self::ChannelEpochAlreadyCreated,
        Self::ChannelEpochChannelMismatch,
        Self::ChannelEpochCounterMismatch,
        Self::BidderPositionChannelEpochMismatch,
        Self::BidderLotPositionMismatch,
        Self::ChannelEpochAlreadyFinalized,
        Self::ChannelEpochNotFinalized,
        Self::BidderPositionClaimed,
        Self::DepositTooSmall,
        Self::WithdrawalTooSmall,
        Self::WithdrawalExceedsPositionCredits,
        Self::EntryWeightOverflow,
        Self::ExitPenaltyOverflow,
        Self::LotSharesOverflow,
        Self::LotSharesRemovedOverflow,
        Self::LotCreditsUnderflow,
        Self::LotSharesUnderflow,
        Self::WithdrawnSharesOverflow,
        Self::CreditsReturnedOverflow,
        Self::CreditsForfeitedUnderflow,
        Self::RewardPotOverflow,
        Self::RewardPotUnderflow,
        Self::CreditBalanceOverflow,
        Self::CreditBalanceUnderflow,
        Self::WeightedSharesOverflow,
        Self::WeightedSharesUnderflow,
        Self::NoSharesToClaim,
        Self::ClaimedSharesOverflow,
        Self::ClaimRewardOverflow,
        Self::ChannelDisabled,
        Self::InvalidTimestamp,
        Self::EpochDurationOutOfRange,
        Self::LotLimitReached,
        Self::EpochDurationTooShort,
        Self::ChannelGroupIndexOverflow,
        Self::NativeClearingMint,
        Self::WithdrawalDestinationIsEscrow,
        Self::ClaimDestinationIsEscrow,
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
            Self::InvalidEpochDuration => "InvalidEpochDuration",
            Self::InvalidRampBonusSlope => "InvalidRampBonusSlope",
            Self::InvalidExitPenaltyExponent => "InvalidExitPenaltyExponent",
            Self::EpochNotStarted => "EpochNotStarted",
            Self::EpochEnded => "EpochEnded",
            Self::EpochNotEnded => "EpochNotEnded",
            Self::EpochEndTimeOverflow => "EpochEndTimeOverflow",
            Self::EpochCounterOverflow => "EpochCounterOverflow",
            Self::EpochChannelGroupMismatch => "EpochChannelGroupMismatch",
            Self::ChannelGroupCounterMismatch => "ChannelGroupCounterMismatch",
            Self::ChannelGroupMismatch => "ChannelGroupMismatch",
            Self::ChannelEpochAlreadyCreated => "ChannelEpochAlreadyCreated",
            Self::ChannelEpochChannelMismatch => "ChannelEpochChannelMismatch",
            Self::ChannelEpochCounterMismatch => "ChannelEpochCounterMismatch",
            Self::BidderPositionChannelEpochMismatch => "BidderPositionChannelEpochMismatch",
            Self::BidderLotPositionMismatch => "BidderLotPositionMismatch",
            Self::ChannelEpochAlreadyFinalized => "ChannelEpochAlreadyFinalized",
            Self::ChannelEpochNotFinalized => "ChannelEpochNotFinalized",
            Self::BidderPositionClaimed => "BidderPositionClaimed",
            Self::DepositTooSmall => "DepositTooSmall",
            Self::WithdrawalTooSmall => "WithdrawalTooSmall",
            Self::WithdrawalExceedsPositionCredits => "WithdrawalExceedsPositionCredits",
            Self::EntryWeightOverflow => "EntryWeightOverflow",
            Self::ExitPenaltyOverflow => "ExitPenaltyOverflow",
            Self::LotSharesOverflow => "LotSharesOverflow",
            Self::LotSharesRemovedOverflow => "LotSharesRemovedOverflow",
            Self::LotCreditsUnderflow => "LotCreditsUnderflow",
            Self::LotSharesUnderflow => "LotSharesUnderflow",
            Self::WithdrawnSharesOverflow => "WithdrawnSharesOverflow",
            Self::CreditsReturnedOverflow => "CreditsReturnedOverflow",
            Self::CreditsForfeitedUnderflow => "CreditsForfeitedUnderflow",
            Self::RewardPotOverflow => "RewardPotOverflow",
            Self::RewardPotUnderflow => "RewardPotUnderflow",
            Self::CreditBalanceOverflow => "CreditBalanceOverflow",
            Self::CreditBalanceUnderflow => "CreditBalanceUnderflow",
            Self::WeightedSharesOverflow => "WeightedSharesOverflow",
            Self::WeightedSharesUnderflow => "WeightedSharesUnderflow",
            Self::NoSharesToClaim => "NoSharesToClaim",
            Self::ClaimedSharesOverflow => "ClaimedSharesOverflow",
            Self::ClaimRewardOverflow => "ClaimRewardOverflow",
            Self::ChannelDisabled => "ChannelDisabled",
            Self::InvalidTimestamp => "InvalidTimestamp",
            Self::EpochDurationOutOfRange => "EpochDurationOutOfRange",
            Self::LotLimitReached => "LotLimitReached",
            Self::EpochDurationTooShort => "EpochDurationTooShort",
            Self::ChannelGroupIndexOverflow => "ChannelGroupIndexOverflow",
            Self::NativeClearingMint => "NativeClearingMint",
            Self::WithdrawalDestinationIsEscrow => "WithdrawalDestinationIsEscrow",
            Self::ClaimDestinationIsEscrow => "ClaimDestinationIsEscrow",
        }
    }

    /// The message the program declares for this error.
    pub fn message(self) -> &'static str {
        match self {
            Self::InvalidEpochDuration => "The epoch duration is zero",
            Self::InvalidRampBonusSlope => "The ramp bonus slope is outside [0, 2]",
            Self::InvalidExitPenaltyExponent => "The exit penalty exponent is outside [1, 4]",
            Self::EpochNotStarted => "The time is before the epoch starts",
            Self::EpochEnded => "The time is at or after the epoch's end",
            Self::EpochNotEnded => "The time is before the epoch's end",
            Self::EpochEndTimeOverflow => "The epoch's start time plus its duration overflows",
            Self::EpochCounterOverflow => "The next epoch counter overflows",
            Self::EpochChannelGroupMismatch => "The epoch belongs to a different channel group",
            Self::ChannelGroupCounterMismatch => {
                "The channel group's current epoch counter differs from the epoch's"
            }
            Self::ChannelGroupMismatch => {
                "The channel and the epoch belong to different channel groups"
            }
            Self::ChannelEpochAlreadyCreated => {
                "The channel already opened a channel epoch for this epoch or a later one"
            }
            Self::ChannelEpochChannelMismatch => "The channel epoch belongs to a different channel",
            Self::ChannelEpochCounterMismatch => {
                "The channel epoch's epoch counter differs from the epoch's"
            }
            Self::BidderPositionChannelEpochMismatch => {
                "The bidder position belongs to a different channel epoch"
            }
            Self::BidderLotPositionMismatch => "A lot belongs to a different bidder position",
            Self::ChannelEpochAlreadyFinalized => "The channel epoch has already been finalized",
            Self::ChannelEpochNotFinalized => {
                "The channel epoch has not been finalized, so it cannot pay claims yet"
            }
            Self::BidderPositionClaimed => "The bidder position has claimed, so it is frozen",
            Self::DepositTooSmall => "The deposit is too small to issue any weighted shares",
            Self::WithdrawalTooSmall => "The withdrawal is zero credits",
            Self::WithdrawalExceedsPositionCredits => {
                "The withdrawal is more than the position's credits"
            }
            Self::EntryWeightOverflow => "The entry weight does not fit the number type",
            Self::ExitPenaltyOverflow => "The exit penalty does not fit the number type",
            Self::LotSharesOverflow => "A new lot's shares do not fit the number type",
            Self::LotSharesRemovedOverflow => {
                "The shares removed from a lot do not fit the number type"
            }
            Self::LotCreditsUnderflow => "A withdrawal takes more credits than the lot holds",
            Self::LotSharesUnderflow => "A withdrawal removes more shares than the lot holds",
            Self::WithdrawnSharesOverflow => "The shares removed across all lots overflow",
            Self::CreditsReturnedOverflow => {
                "The credits returned after the exit penalty do not fit the number type"
            }
            Self::CreditsForfeitedUnderflow => "The credits returned exceed the credits withdrawn",
            Self::RewardPotOverflow => "The reward pot balance overflows",
            Self::RewardPotUnderflow => "A claim pays more reward than the reward pot holds",
            Self::CreditBalanceOverflow => "The clearing credits balance overflows",
            Self::CreditBalanceUnderflow => {
                "A withdrawal takes more credits than the clearing credits balance holds"
            }
            Self::WeightedSharesOverflow => "The weighted shares balance overflows",
            Self::WeightedSharesUnderflow => {
                "A withdrawal or claim removes more shares than the weighted shares balance holds"
            }
            Self::NoSharesToClaim => "The position has no shares to claim",
            Self::ClaimedSharesOverflow => {
                "The position's shares overflow when summed across its lots"
            }
            Self::ClaimRewardOverflow => "The claim's reward does not fit the number type",
            Self::ChannelDisabled => "The channel is disabled",
            Self::InvalidTimestamp => "The clock timestamp is negative",
            Self::EpochDurationOutOfRange => {
                "The epoch duration exceeds the stored u32 seconds range"
            }
            Self::LotLimitReached => "The bidder position has reached its lot limit",
            Self::EpochDurationTooShort => {
                "The epoch duration must be at least one hour (3600 seconds)"
            }
            Self::ChannelGroupIndexOverflow => {
                "The organization's next channel group index overflows"
            }
            Self::NativeClearingMint => "The native mint cannot be used for clearing credits",
            Self::WithdrawalDestinationIsEscrow => {
                "The withdrawal destination cannot be the clearing credits escrow"
            }
            Self::ClaimDestinationIsEscrow => {
                "The claim destination cannot be the reward pot escrow"
            }
        }
    }
}

impl core::fmt::Display for MayfRewardClearingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ({}): {}", self.name(), *self as u32, self.message())
    }
}

impl std::error::Error for MayfRewardClearingError {}

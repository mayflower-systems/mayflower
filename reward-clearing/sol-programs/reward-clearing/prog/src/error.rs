use anchor_lang::prelude::*;
use mayf_reward_clearing::RewardClearingError;

/// Reward-clearing domain failures and Solana host policy errors.
#[error_code]
pub enum RewardClearingProgramError {
    // Epoch terms
    #[msg("The epoch duration is zero")]
    InvalidEpochDuration,
    #[msg("The ramp bonus slope is outside [0, 2]")]
    InvalidRampBonusSlope,
    #[msg("The exit penalty exponent is outside [1, 4]")]
    InvalidExitPenaltyExponent,

    // Epoch timing
    #[msg("The time is before the epoch starts")]
    EpochNotStarted,
    #[msg("The time is at or after the epoch's end")]
    EpochEnded,
    #[msg("The time is before the epoch's end")]
    EpochNotEnded,
    #[msg("The epoch's start time plus its duration overflows")]
    EpochEndTimeOverflow,
    #[msg("The next epoch counter overflows")]
    EpochCounterOverflow,

    // Models that do not belong together
    #[msg("The epoch belongs to a different channel group")]
    EpochChannelGroupMismatch,
    #[msg("The channel group's current epoch counter differs from the epoch's")]
    ChannelGroupCounterMismatch,
    #[msg("The channel and the epoch belong to different channel groups")]
    ChannelGroupMismatch,
    #[msg("The channel already opened a channel epoch for this epoch or a later one")]
    ChannelEpochAlreadyCreated,
    #[msg("The channel epoch belongs to a different channel")]
    ChannelEpochChannelMismatch,
    #[msg("The channel epoch's epoch counter differs from the epoch's")]
    ChannelEpochCounterMismatch,
    #[msg("The bidder position belongs to a different channel epoch")]
    BidderPositionChannelEpochMismatch,
    #[msg("A lot belongs to a different bidder position")]
    BidderLotPositionMismatch,

    // Lifecycle
    #[msg("The channel epoch has already been finalized")]
    ChannelEpochAlreadyFinalized,
    #[msg("The channel epoch has not been finalized, so it cannot pay claims yet")]
    ChannelEpochNotFinalized,
    #[msg("The bidder position has claimed, so it is frozen")]
    BidderPositionClaimed,

    // Requested amounts
    #[msg("The deposit is too small to issue any weighted shares")]
    DepositTooSmall,
    #[msg("The withdrawal is zero credits")]
    WithdrawalTooSmall,
    #[msg("The withdrawal is more than the position's credits")]
    WithdrawalExceedsPositionCredits,

    // Curves
    #[msg("The entry weight does not fit the number type")]
    EntryWeightOverflow,
    #[msg("The exit penalty does not fit the number type")]
    ExitPenaltyOverflow,

    // Lots
    #[msg("A new lot's shares do not fit the number type")]
    LotSharesOverflow,
    #[msg("The shares removed from a lot do not fit the number type")]
    LotSharesRemovedOverflow,
    #[msg("A withdrawal takes more credits than the lot holds")]
    LotCreditsUnderflow,
    #[msg("A withdrawal removes more shares than the lot holds")]
    LotSharesUnderflow,
    #[msg("The shares removed across all lots overflow")]
    WithdrawnSharesOverflow,

    // Withdrawn credits
    #[msg("The credits returned after the exit penalty do not fit the number type")]
    CreditsReturnedOverflow,
    #[msg("The credits returned exceed the credits withdrawn")]
    CreditsForfeitedUnderflow,

    // Channel epoch balances
    #[msg("The reward pot balance overflows")]
    RewardPotOverflow,
    #[msg("A claim pays more reward than the reward pot holds")]
    RewardPotUnderflow,
    #[msg("The clearing credits balance overflows")]
    CreditBalanceOverflow,
    #[msg("A withdrawal takes more credits than the clearing credits balance holds")]
    CreditBalanceUnderflow,
    #[msg("The weighted shares balance overflows")]
    WeightedSharesOverflow,
    #[msg("A withdrawal or claim removes more shares than the weighted shares balance holds")]
    WeightedSharesUnderflow,

    // Claims
    #[msg("The position has no shares to claim")]
    NoSharesToClaim,
    #[msg("The position's shares overflow when summed across its lots")]
    ClaimedSharesOverflow,
    #[msg("The claim's reward does not fit the number type")]
    ClaimRewardOverflow,

    #[msg("The channel is disabled")]
    ChannelDisabled,
    #[msg("The clock timestamp is negative")]
    InvalidTimestamp,
    #[msg("The epoch duration exceeds the stored u32 seconds range")]
    EpochDurationOutOfRange,
    #[msg("The bidder position has reached its lot limit")]
    LotLimitReached,
    #[msg("The epoch duration must be at least one hour (3600 seconds)")]
    EpochDurationTooShort,
    #[msg("The organization's next channel group index overflows")]
    ChannelGroupIndexOverflow,

    // Token custody constraints
    #[msg("The native mint cannot be used for clearing credits")]
    NativeClearingMint,
    #[msg("The withdrawal destination cannot be the clearing credits escrow")]
    WithdrawalDestinationIsEscrow,
    #[msg("The claim destination cannot be the reward pot escrow")]
    ClaimDestinationIsEscrow,
}

impl From<RewardClearingError> for RewardClearingProgramError {
    fn from(error: RewardClearingError) -> Self {
        match error {
            RewardClearingError::LotLimitReached => Self::LotLimitReached,
            RewardClearingError::ChannelDisabled => Self::ChannelDisabled,
            RewardClearingError::InvalidEpochDuration => Self::InvalidEpochDuration,
            RewardClearingError::EpochDurationTooShort => Self::EpochDurationTooShort,
            RewardClearingError::InvalidRampBonusSlope => Self::InvalidRampBonusSlope,
            RewardClearingError::InvalidExitPenaltyExponent => Self::InvalidExitPenaltyExponent,
            RewardClearingError::EpochNotStarted => Self::EpochNotStarted,
            RewardClearingError::EpochEnded => Self::EpochEnded,
            RewardClearingError::EpochNotEnded => Self::EpochNotEnded,
            RewardClearingError::EpochEndTimeOverflow => Self::EpochEndTimeOverflow,
            RewardClearingError::EpochCounterOverflow => Self::EpochCounterOverflow,
            RewardClearingError::EpochChannelGroupMismatch => Self::EpochChannelGroupMismatch,
            RewardClearingError::ChannelGroupCounterMismatch => Self::ChannelGroupCounterMismatch,
            RewardClearingError::ChannelGroupMismatch => Self::ChannelGroupMismatch,
            RewardClearingError::ChannelEpochAlreadyCreated => Self::ChannelEpochAlreadyCreated,
            RewardClearingError::ChannelEpochChannelMismatch => Self::ChannelEpochChannelMismatch,
            RewardClearingError::ChannelEpochCounterMismatch => Self::ChannelEpochCounterMismatch,
            RewardClearingError::BidderPositionChannelEpochMismatch => {
                Self::BidderPositionChannelEpochMismatch
            }
            RewardClearingError::BidderLotPositionMismatch => Self::BidderLotPositionMismatch,
            RewardClearingError::ChannelEpochAlreadyFinalized => Self::ChannelEpochAlreadyFinalized,
            RewardClearingError::ChannelEpochNotFinalized => Self::ChannelEpochNotFinalized,
            RewardClearingError::BidderPositionClaimed => Self::BidderPositionClaimed,
            RewardClearingError::DepositTooSmall => Self::DepositTooSmall,
            RewardClearingError::WithdrawalTooSmall => Self::WithdrawalTooSmall,
            RewardClearingError::WithdrawalExceedsPositionCredits => {
                Self::WithdrawalExceedsPositionCredits
            }
            RewardClearingError::EntryWeightOverflow => Self::EntryWeightOverflow,
            RewardClearingError::ExitPenaltyOverflow => Self::ExitPenaltyOverflow,
            RewardClearingError::LotSharesOverflow => Self::LotSharesOverflow,
            RewardClearingError::LotSharesRemovedOverflow => Self::LotSharesRemovedOverflow,
            RewardClearingError::LotCreditsUnderflow => Self::LotCreditsUnderflow,
            RewardClearingError::LotSharesUnderflow => Self::LotSharesUnderflow,
            RewardClearingError::WithdrawnSharesOverflow => Self::WithdrawnSharesOverflow,
            RewardClearingError::CreditsReturnedOverflow => Self::CreditsReturnedOverflow,
            RewardClearingError::CreditsForfeitedUnderflow => Self::CreditsForfeitedUnderflow,
            RewardClearingError::RewardPotOverflow => Self::RewardPotOverflow,
            RewardClearingError::RewardPotUnderflow => Self::RewardPotUnderflow,
            RewardClearingError::CreditBalanceOverflow => Self::CreditBalanceOverflow,
            RewardClearingError::CreditBalanceUnderflow => Self::CreditBalanceUnderflow,
            RewardClearingError::WeightedSharesOverflow => Self::WeightedSharesOverflow,
            RewardClearingError::WeightedSharesUnderflow => Self::WeightedSharesUnderflow,
            RewardClearingError::NoSharesToClaim => Self::NoSharesToClaim,
            RewardClearingError::ClaimedSharesOverflow => Self::ClaimedSharesOverflow,
            RewardClearingError::ClaimRewardOverflow => Self::ClaimRewardOverflow,
        }
    }
}

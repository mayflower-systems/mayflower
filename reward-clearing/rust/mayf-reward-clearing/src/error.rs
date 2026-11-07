/// Every way a reward-clearing action can fail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewardClearingError {
    // Epoch terms
    /// The epoch duration is zero
    InvalidEpochDuration,
    /// The ramp bonus slope is outside [0, 2]
    InvalidRampBonusSlope,
    /// The exit penalty exponent is outside [1, 4]
    InvalidExitPenaltyExponent,

    // Epoch timing
    /// The time is before the epoch starts
    EpochNotStarted,
    /// The time is at or after the epoch's end
    EpochEnded,
    /// The time is before the epoch's end
    EpochNotEnded,
    /// The epoch's start time plus its duration overflows
    EpochEndTimeOverflow,
    /// The next epoch counter overflows
    EpochCounterOverflow,

    // Models that do not belong together
    /// The channel group's current epoch counter differs from the epoch's
    ChannelGroupCounterMismatch,
    /// The channel epoch's epoch counter differs from the epoch's
    ChannelEpochCounterMismatch,
    /// The bidder position belongs to a different channel epoch
    BidderPositionChannelEpochMismatch,
    /// The final snapshot belongs to a different channel epoch
    ChannelEpochFinalMismatch,

    // Requested amounts
    /// The deposit is too small to issue any weighted shares
    DepositTooSmall,
    /// The withdrawal is zero credits
    WithdrawalTooSmall,
    /// The withdrawal is more than the position's credits
    WithdrawalExceedsPositionCredits,

    // Curves
    /// The entry weight does not fit the number type
    EntryWeightOverflow,
    /// The exit penalty does not fit the number type
    ExitPenaltyOverflow,

    // Lots
    /// A new lot's shares do not fit the number type
    LotSharesOverflow,
    /// The shares removed from a lot do not fit the number type
    LotSharesRemovedOverflow,
    /// A withdrawal takes more credits than the lot holds
    LotCreditsUnderflow,
    /// A withdrawal removes more shares than the lot holds
    LotSharesUnderflow,
    /// The shares removed across all lots overflow
    WithdrawnSharesOverflow,

    // Withdrawn credits
    /// The credits returned after the exit penalty do not fit the number type
    CreditsReturnedOverflow,
    /// The credits returned exceed the credits withdrawn
    CreditsForfeitedUnderflow,

    // Channel epoch balances
    /// The reward pot balance overflows
    RewardPotOverflow,
    /// The clearing credits balance overflows
    CreditBalanceOverflow,
    /// A withdrawal takes more credits than the clearing credits balance holds
    CreditBalanceUnderflow,
    /// The weighted shares balance overflows
    WeightedSharesOverflow,
    /// A withdrawal removes more shares than the weighted shares balance holds
    WeightedSharesUnderflow,

    // Claims
    /// The position has no shares left to claim
    NoSharesToClaim,
    /// The position's shares overflow when summed across its lots
    ClaimedSharesOverflow,
    /// A claim divides by a weighted shares balance of zero
    ZeroWeightedShares,
    /// The claim's reward does not fit the number type
    ClaimRewardOverflow,
    /// A claim retires more shares than the final snapshot has unclaimed
    UnclaimedSharesUnderflow,
}

pub type Result<T> = core::result::Result<T, RewardClearingError>;

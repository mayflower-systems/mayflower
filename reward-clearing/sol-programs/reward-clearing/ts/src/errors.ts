// Generated from an Anchor IDL. Do not edit.

// program errors ----------------------------------------------------

export const MAYF_REWARD_CLEARING_ERRORS = {
  InvalidEpochDuration: { code: 6000, message: "The epoch duration is zero" },
  InvalidRampBonusSlope: {
    code: 6001,
    message: "The ramp bonus slope is outside [0, 2]",
  },
  InvalidExitPenaltyExponent: {
    code: 6002,
    message: "The exit penalty exponent is outside [1, 4]",
  },
  EpochNotStarted: {
    code: 6003,
    message: "The time is before the epoch starts",
  },
  EpochEnded: {
    code: 6004,
    message: "The time is at or after the epoch's end",
  },
  EpochNotEnded: { code: 6005, message: "The time is before the epoch's end" },
  EpochEndTimeOverflow: {
    code: 6006,
    message: "The epoch's start time plus its duration overflows",
  },
  EpochCounterOverflow: {
    code: 6007,
    message: "The next epoch counter overflows",
  },
  EpochChannelGroupMismatch: {
    code: 6008,
    message: "The epoch belongs to a different channel group",
  },
  ChannelGroupCounterMismatch: {
    code: 6009,
    message:
      "The channel group's current epoch counter differs from the epoch's",
  },
  ChannelGroupMismatch: {
    code: 6010,
    message: "The channel and the epoch belong to different channel groups",
  },
  ChannelEpochAlreadyCreated: {
    code: 6011,
    message:
      "The channel already opened a channel epoch for this epoch or a later one",
  },
  ChannelEpochChannelMismatch: {
    code: 6012,
    message: "The channel epoch belongs to a different channel",
  },
  ChannelEpochCounterMismatch: {
    code: 6013,
    message: "The channel epoch's epoch counter differs from the epoch's",
  },
  BidderPositionChannelEpochMismatch: {
    code: 6014,
    message: "The bidder position belongs to a different channel epoch",
  },
  BidderLotPositionMismatch: {
    code: 6015,
    message: "A lot belongs to a different bidder position",
  },
  ChannelEpochAlreadyFinalized: {
    code: 6016,
    message: "The channel epoch has already been finalized",
  },
  ChannelEpochNotFinalized: {
    code: 6017,
    message:
      "The channel epoch has not been finalized, so it cannot pay claims yet",
  },
  BidderPositionClaimed: {
    code: 6018,
    message: "The bidder position has claimed, so it is frozen",
  },
  DepositTooSmall: {
    code: 6019,
    message: "The deposit is too small to issue any weighted shares",
  },
  WithdrawalTooSmall: { code: 6020, message: "The withdrawal is zero credits" },
  WithdrawalExceedsPositionCredits: {
    code: 6021,
    message: "The withdrawal is more than the position's credits",
  },
  EntryWeightOverflow: {
    code: 6022,
    message: "The entry weight does not fit the number type",
  },
  ExitPenaltyOverflow: {
    code: 6023,
    message: "The exit penalty does not fit the number type",
  },
  LotSharesOverflow: {
    code: 6024,
    message: "A new lot's shares do not fit the number type",
  },
  LotSharesRemovedOverflow: {
    code: 6025,
    message: "The shares removed from a lot do not fit the number type",
  },
  LotCreditsUnderflow: {
    code: 6026,
    message: "A withdrawal takes more credits than the lot holds",
  },
  LotSharesUnderflow: {
    code: 6027,
    message: "A withdrawal removes more shares than the lot holds",
  },
  WithdrawnSharesOverflow: {
    code: 6028,
    message: "The shares removed across all lots overflow",
  },
  CreditsReturnedOverflow: {
    code: 6029,
    message:
      "The credits returned after the exit penalty do not fit the number type",
  },
  CreditsForfeitedUnderflow: {
    code: 6030,
    message: "The credits returned exceed the credits withdrawn",
  },
  RewardPotOverflow: {
    code: 6031,
    message: "The reward pot balance overflows",
  },
  RewardPotUnderflow: {
    code: 6032,
    message: "A claim pays more reward than the reward pot holds",
  },
  CreditBalanceOverflow: {
    code: 6033,
    message: "The clearing credits balance overflows",
  },
  CreditBalanceUnderflow: {
    code: 6034,
    message:
      "A withdrawal takes more credits than the clearing credits balance holds",
  },
  WeightedSharesOverflow: {
    code: 6035,
    message: "The weighted shares balance overflows",
  },
  WeightedSharesUnderflow: {
    code: 6036,
    message:
      "A withdrawal or claim removes more shares than the weighted shares balance holds",
  },
  NoSharesToClaim: {
    code: 6037,
    message: "The position has no shares to claim",
  },
  ClaimedSharesOverflow: {
    code: 6038,
    message: "The position's shares overflow when summed across its lots",
  },
  ClaimRewardOverflow: {
    code: 6039,
    message: "The claim's reward does not fit the number type",
  },
  ChannelDisabled: { code: 6040, message: "The channel is disabled" },
  InvalidTimestamp: { code: 6041, message: "The clock timestamp is negative" },
  EpochDurationOutOfRange: {
    code: 6042,
    message: "The epoch duration exceeds the stored u32 seconds range",
  },
  LotLimitReached: {
    code: 6043,
    message: "The bidder position has reached its lot limit",
  },
  EpochDurationTooShort: {
    code: 6044,
    message: "The epoch duration must be at least one hour (3600 seconds)",
  },
  ChannelGroupIndexOverflow: {
    code: 6045,
    message: "The organization's next channel group index overflows",
  },
} as const

export type MayfRewardClearingErrorName =
  keyof typeof MAYF_REWARD_CLEARING_ERRORS

export interface MayfRewardClearingErrorDetails {
  name: MayfRewardClearingErrorName
  code: number
  message: string
}

export const getMayfRewardClearingError = (
  code: number,
): MayfRewardClearingErrorDetails | null => {
  switch (code) {
    case 6000:
      return {
        name: "InvalidEpochDuration",
        ...MAYF_REWARD_CLEARING_ERRORS.InvalidEpochDuration,
      }
    case 6001:
      return {
        name: "InvalidRampBonusSlope",
        ...MAYF_REWARD_CLEARING_ERRORS.InvalidRampBonusSlope,
      }
    case 6002:
      return {
        name: "InvalidExitPenaltyExponent",
        ...MAYF_REWARD_CLEARING_ERRORS.InvalidExitPenaltyExponent,
      }
    case 6003:
      return {
        name: "EpochNotStarted",
        ...MAYF_REWARD_CLEARING_ERRORS.EpochNotStarted,
      }
    case 6004:
      return { name: "EpochEnded", ...MAYF_REWARD_CLEARING_ERRORS.EpochEnded }
    case 6005:
      return {
        name: "EpochNotEnded",
        ...MAYF_REWARD_CLEARING_ERRORS.EpochNotEnded,
      }
    case 6006:
      return {
        name: "EpochEndTimeOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.EpochEndTimeOverflow,
      }
    case 6007:
      return {
        name: "EpochCounterOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.EpochCounterOverflow,
      }
    case 6008:
      return {
        name: "EpochChannelGroupMismatch",
        ...MAYF_REWARD_CLEARING_ERRORS.EpochChannelGroupMismatch,
      }
    case 6009:
      return {
        name: "ChannelGroupCounterMismatch",
        ...MAYF_REWARD_CLEARING_ERRORS.ChannelGroupCounterMismatch,
      }
    case 6010:
      return {
        name: "ChannelGroupMismatch",
        ...MAYF_REWARD_CLEARING_ERRORS.ChannelGroupMismatch,
      }
    case 6011:
      return {
        name: "ChannelEpochAlreadyCreated",
        ...MAYF_REWARD_CLEARING_ERRORS.ChannelEpochAlreadyCreated,
      }
    case 6012:
      return {
        name: "ChannelEpochChannelMismatch",
        ...MAYF_REWARD_CLEARING_ERRORS.ChannelEpochChannelMismatch,
      }
    case 6013:
      return {
        name: "ChannelEpochCounterMismatch",
        ...MAYF_REWARD_CLEARING_ERRORS.ChannelEpochCounterMismatch,
      }
    case 6014:
      return {
        name: "BidderPositionChannelEpochMismatch",
        ...MAYF_REWARD_CLEARING_ERRORS.BidderPositionChannelEpochMismatch,
      }
    case 6015:
      return {
        name: "BidderLotPositionMismatch",
        ...MAYF_REWARD_CLEARING_ERRORS.BidderLotPositionMismatch,
      }
    case 6016:
      return {
        name: "ChannelEpochAlreadyFinalized",
        ...MAYF_REWARD_CLEARING_ERRORS.ChannelEpochAlreadyFinalized,
      }
    case 6017:
      return {
        name: "ChannelEpochNotFinalized",
        ...MAYF_REWARD_CLEARING_ERRORS.ChannelEpochNotFinalized,
      }
    case 6018:
      return {
        name: "BidderPositionClaimed",
        ...MAYF_REWARD_CLEARING_ERRORS.BidderPositionClaimed,
      }
    case 6019:
      return {
        name: "DepositTooSmall",
        ...MAYF_REWARD_CLEARING_ERRORS.DepositTooSmall,
      }
    case 6020:
      return {
        name: "WithdrawalTooSmall",
        ...MAYF_REWARD_CLEARING_ERRORS.WithdrawalTooSmall,
      }
    case 6021:
      return {
        name: "WithdrawalExceedsPositionCredits",
        ...MAYF_REWARD_CLEARING_ERRORS.WithdrawalExceedsPositionCredits,
      }
    case 6022:
      return {
        name: "EntryWeightOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.EntryWeightOverflow,
      }
    case 6023:
      return {
        name: "ExitPenaltyOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.ExitPenaltyOverflow,
      }
    case 6024:
      return {
        name: "LotSharesOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.LotSharesOverflow,
      }
    case 6025:
      return {
        name: "LotSharesRemovedOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.LotSharesRemovedOverflow,
      }
    case 6026:
      return {
        name: "LotCreditsUnderflow",
        ...MAYF_REWARD_CLEARING_ERRORS.LotCreditsUnderflow,
      }
    case 6027:
      return {
        name: "LotSharesUnderflow",
        ...MAYF_REWARD_CLEARING_ERRORS.LotSharesUnderflow,
      }
    case 6028:
      return {
        name: "WithdrawnSharesOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.WithdrawnSharesOverflow,
      }
    case 6029:
      return {
        name: "CreditsReturnedOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.CreditsReturnedOverflow,
      }
    case 6030:
      return {
        name: "CreditsForfeitedUnderflow",
        ...MAYF_REWARD_CLEARING_ERRORS.CreditsForfeitedUnderflow,
      }
    case 6031:
      return {
        name: "RewardPotOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.RewardPotOverflow,
      }
    case 6032:
      return {
        name: "RewardPotUnderflow",
        ...MAYF_REWARD_CLEARING_ERRORS.RewardPotUnderflow,
      }
    case 6033:
      return {
        name: "CreditBalanceOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.CreditBalanceOverflow,
      }
    case 6034:
      return {
        name: "CreditBalanceUnderflow",
        ...MAYF_REWARD_CLEARING_ERRORS.CreditBalanceUnderflow,
      }
    case 6035:
      return {
        name: "WeightedSharesOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.WeightedSharesOverflow,
      }
    case 6036:
      return {
        name: "WeightedSharesUnderflow",
        ...MAYF_REWARD_CLEARING_ERRORS.WeightedSharesUnderflow,
      }
    case 6037:
      return {
        name: "NoSharesToClaim",
        ...MAYF_REWARD_CLEARING_ERRORS.NoSharesToClaim,
      }
    case 6038:
      return {
        name: "ClaimedSharesOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.ClaimedSharesOverflow,
      }
    case 6039:
      return {
        name: "ClaimRewardOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.ClaimRewardOverflow,
      }
    case 6040:
      return {
        name: "ChannelDisabled",
        ...MAYF_REWARD_CLEARING_ERRORS.ChannelDisabled,
      }
    case 6041:
      return {
        name: "InvalidTimestamp",
        ...MAYF_REWARD_CLEARING_ERRORS.InvalidTimestamp,
      }
    case 6042:
      return {
        name: "EpochDurationOutOfRange",
        ...MAYF_REWARD_CLEARING_ERRORS.EpochDurationOutOfRange,
      }
    case 6043:
      return {
        name: "LotLimitReached",
        ...MAYF_REWARD_CLEARING_ERRORS.LotLimitReached,
      }
    case 6044:
      return {
        name: "EpochDurationTooShort",
        ...MAYF_REWARD_CLEARING_ERRORS.EpochDurationTooShort,
      }
    case 6045:
      return {
        name: "ChannelGroupIndexOverflow",
        ...MAYF_REWARD_CLEARING_ERRORS.ChannelGroupIndexOverflow,
      }
    default:
      return null
  }
}

export class MayfRewardClearingProgramError extends Error {
  readonly code: number
  readonly errorName: MayfRewardClearingErrorName

  constructor(details: MayfRewardClearingErrorDetails) {
    super(`${details.name} (${details.code}): ${details.message}`)
    this.name = "MayfRewardClearingProgramError"
    this.code = details.code
    this.errorName = details.name
  }

  static fromCode(code: number): MayfRewardClearingProgramError | null {
    const details = getMayfRewardClearingError(code)
    return details === null ? null : new MayfRewardClearingProgramError(details)
  }
}

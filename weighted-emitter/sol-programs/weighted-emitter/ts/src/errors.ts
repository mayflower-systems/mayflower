// Generated from an Anchor IDL. Do not edit.

// program errors ----------------------------------------------------

export const MAYF_WEIGHTED_EMITTER_ERRORS = {
  TimeWentBack: { code: 6000, message: "The clock moved backwards" },
  GaugeEmitterMismatch: {
    code: 6001,
    message: "The gauge belongs to a different emitter",
  },
  InvalidState: {
    code: 6002,
    message: "The supplied accounting state is inconsistent",
  },
  InsufficientPrecision: {
    code: 6003,
    message: "The accounting number has insufficient precision",
  },
  ArithmeticOverflow: {
    code: 6004,
    message: "An accounting amount overflowed",
  },
  InvalidTimestamp: { code: 6005, message: "The clock timestamp is negative" },
  NativeEmissionMint: {
    code: 6006,
    message: "The native mint cannot be an emission mint",
  },
  InvalidMintAuthority: {
    code: 6007,
    message:
      "The emission mint authority must be the emitter's mint authority PDA",
  },
  ZeroAmount: {
    code: 6008,
    message: "A vote deposit or withdrawal must be positive",
  },
  InsufficientDeposit: {
    code: 6009,
    message: "The withdrawal exceeds this position's recorded deposit",
  },
  EscrowUnderfunded: {
    code: 6010,
    message: "The vote escrow does not back the recorded deposit",
  },
  DepositSourceIsEscrow: {
    code: 6011,
    message: "The deposit source cannot be its own vote escrow",
  },
  WithdrawalDestinationIsEscrow: {
    code: 6012,
    message: "The withdrawal destination cannot be its own vote escrow",
  },
} as const

export type MayfWeightedEmitterErrorName =
  keyof typeof MAYF_WEIGHTED_EMITTER_ERRORS

export interface MayfWeightedEmitterErrorDetails {
  name: MayfWeightedEmitterErrorName
  code: number
  message: string
}

export const getMayfWeightedEmitterError = (
  code: number,
): MayfWeightedEmitterErrorDetails | null => {
  switch (code) {
    case 6000:
      return {
        name: "TimeWentBack",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.TimeWentBack,
      }
    case 6001:
      return {
        name: "GaugeEmitterMismatch",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.GaugeEmitterMismatch,
      }
    case 6002:
      return {
        name: "InvalidState",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.InvalidState,
      }
    case 6003:
      return {
        name: "InsufficientPrecision",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.InsufficientPrecision,
      }
    case 6004:
      return {
        name: "ArithmeticOverflow",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.ArithmeticOverflow,
      }
    case 6005:
      return {
        name: "InvalidTimestamp",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.InvalidTimestamp,
      }
    case 6006:
      return {
        name: "NativeEmissionMint",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.NativeEmissionMint,
      }
    case 6007:
      return {
        name: "InvalidMintAuthority",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.InvalidMintAuthority,
      }
    case 6008:
      return { name: "ZeroAmount", ...MAYF_WEIGHTED_EMITTER_ERRORS.ZeroAmount }
    case 6009:
      return {
        name: "InsufficientDeposit",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.InsufficientDeposit,
      }
    case 6010:
      return {
        name: "EscrowUnderfunded",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.EscrowUnderfunded,
      }
    case 6011:
      return {
        name: "DepositSourceIsEscrow",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.DepositSourceIsEscrow,
      }
    case 6012:
      return {
        name: "WithdrawalDestinationIsEscrow",
        ...MAYF_WEIGHTED_EMITTER_ERRORS.WithdrawalDestinationIsEscrow,
      }
    default:
      return null
  }
}

export class MayfWeightedEmitterProgramError extends Error {
  readonly code: number
  readonly errorName: MayfWeightedEmitterErrorName

  constructor(details: MayfWeightedEmitterErrorDetails) {
    super(`${details.name} (${details.code}): ${details.message}`)
    this.name = "MayfWeightedEmitterProgramError"
    this.code = details.code
    this.errorName = details.name
  }

  static fromCode(code: number): MayfWeightedEmitterProgramError | null {
    const details = getMayfWeightedEmitterError(code)
    return details === null ? null : (
        new MayfWeightedEmitterProgramError(details)
      )
  }
}

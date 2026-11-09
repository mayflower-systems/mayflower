// Generated from an Anchor IDL. Do not edit.

// program errors ----------------------------------------------------

export const MAYF_AVM_ENGINE_LINEAR_ERRORS = {
  InvalidSlope: {
    code: 6000,
    message: "The characteristic slope must be positive",
  },
  BatchTooLarge: {
    code: 6001,
    message: "The span responses would exceed Solana's return-data limit",
  },
  ArithmeticFailure: {
    code: 6002,
    message: "The span or its result exceeds the supported fixed-point range",
  },
} as const

export type MayfAvmEngineLinearErrorName =
  keyof typeof MAYF_AVM_ENGINE_LINEAR_ERRORS

export interface MayfAvmEngineLinearErrorDetails {
  name: MayfAvmEngineLinearErrorName
  code: number
  message: string
}

export const getMayfAvmEngineLinearError = (
  code: number,
): MayfAvmEngineLinearErrorDetails | null => {
  switch (code) {
    case 6000:
      return {
        name: "InvalidSlope",
        ...MAYF_AVM_ENGINE_LINEAR_ERRORS.InvalidSlope,
      }
    case 6001:
      return {
        name: "BatchTooLarge",
        ...MAYF_AVM_ENGINE_LINEAR_ERRORS.BatchTooLarge,
      }
    case 6002:
      return {
        name: "ArithmeticFailure",
        ...MAYF_AVM_ENGINE_LINEAR_ERRORS.ArithmeticFailure,
      }
    default:
      return null
  }
}

export class MayfAvmEngineLinearProgramError extends Error {
  readonly code: number
  readonly errorName: MayfAvmEngineLinearErrorName

  constructor(details: MayfAvmEngineLinearErrorDetails) {
    super(`${details.name} (${details.code}): ${details.message}`)
    this.name = "MayfAvmEngineLinearProgramError"
    this.code = details.code
    this.errorName = details.name
  }

  static fromCode(code: number): MayfAvmEngineLinearProgramError | null {
    const details = getMayfAvmEngineLinearError(code)
    return details === null ? null : (
        new MayfAvmEngineLinearProgramError(details)
      )
  }
}

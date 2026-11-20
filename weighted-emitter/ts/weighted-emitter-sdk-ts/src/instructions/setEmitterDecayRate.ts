import { createSetEmitterDecayRateInstruction } from "@mayflower-sys/weighted-emitter-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import type { BigDecimal } from "../models/BigDecimal.ts"
import { instructionRate, toWeb3Instruction } from "./shared.ts"

export interface SetEmitterDecayRateParams {
  authority: PublicKey
  emitter: PublicKey
  annualDecayRate: BigDecimal
}

export const setEmitterDecayRateInstruction = (
  programId: PublicKey,
  params: SetEmitterDecayRateParams,
): TransactionInstruction =>
  toWeb3Instruction(
    createSetEmitterDecayRateInstruction(
      programId.toBytes(),
      {
        authority: params.authority.toBytes(),
        emitter: params.emitter.toBytes(),
      },
      {
        args: {
          annualDecayRateQ64f64: instructionRate(
            params.annualDecayRate,
            "annualDecayRate",
          ),
        },
      },
    ),
  )

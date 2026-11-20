import { createCreateEmitterInstruction } from "@mayflower-sys/weighted-emitter-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import type { BigDecimal } from "../models/BigDecimal.ts"
import { createWeightedEmitterPda } from "../pda.ts"
import { instructionRate, toWeb3Instruction } from "./shared.ts"

export interface CreateEmitterParams {
  authority: PublicKey
  payer: PublicKey
  /** A new keypair account; its keypair must sign the transaction. */
  emitter: PublicKey
  voteMint: PublicKey
  /** An existing mint whose mint authority is pda.mintAuthority(emitter). */
  emissionMint: PublicKey
  /** Emission token base units per second. */
  emissionRate: BigDecimal
  annualDecayRate: BigDecimal
}

export const createEmitterInstruction = (
  programId: PublicKey,
  params: CreateEmitterParams,
): TransactionInstruction => {
  const pda = createWeightedEmitterPda(programId)
  return toWeb3Instruction(
    createCreateEmitterInstruction(
      programId.toBytes(),
      {
        payer: params.payer.toBytes(),
        authority: params.authority.toBytes(),
        emitter: params.emitter.toBytes(),
        voteMint: params.voteMint.toBytes(),
        emissionMint: params.emissionMint.toBytes(),
        mintAuthority: pda.mintAuthority(params.emitter).toBytes(),
      },
      {
        args: {
          emissionRateQ64f64: instructionRate(
            params.emissionRate,
            "emissionRate",
          ),
          annualDecayRateQ64f64: instructionRate(
            params.annualDecayRate,
            "annualDecayRate",
          ),
        },
      },
    ),
  )
}

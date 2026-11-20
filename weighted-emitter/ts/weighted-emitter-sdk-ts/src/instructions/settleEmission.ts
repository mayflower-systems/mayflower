import { createSettleEmissionInstruction } from "@mayflower-sys/weighted-emitter-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createWeightedEmitterPda } from "../pda.ts"
import { toWeb3Instruction } from "./shared.ts"

export interface SettleEmissionParams {
  authority: PublicKey
  emitter: PublicKey
  gauge: PublicKey
  emissionMint: PublicKey
  /** Any token account for emissionMint; its owner is unrestricted. */
  emissionDestination: PublicKey
}

export const settleEmissionInstruction = (
  programId: PublicKey,
  params: SettleEmissionParams,
): TransactionInstruction => {
  const pda = createWeightedEmitterPda(programId)
  return toWeb3Instruction(
    createSettleEmissionInstruction(programId.toBytes(), {
      authority: params.authority.toBytes(),
      emitter: params.emitter.toBytes(),
      gauge: params.gauge.toBytes(),
      emissionMint: params.emissionMint.toBytes(),
      mintAuthority: pda.mintAuthority(params.emitter).toBytes(),
      emissionDestination: params.emissionDestination.toBytes(),
    }),
  )
}

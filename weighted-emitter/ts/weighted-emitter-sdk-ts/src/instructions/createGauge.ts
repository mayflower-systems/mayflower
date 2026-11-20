import { createCreateGaugeInstruction } from "@mayflower-sys/weighted-emitter-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { toWeb3Instruction } from "./shared.ts"

export interface CreateGaugeParams {
  authority: PublicKey
  payer: PublicKey
  emitter: PublicKey
  /** A new keypair account; its keypair must sign the transaction. */
  gauge: PublicKey
}

export const createGaugeInstruction = (
  programId: PublicKey,
  params: CreateGaugeParams,
): TransactionInstruction =>
  toWeb3Instruction(
    createCreateGaugeInstruction(programId.toBytes(), {
      payer: params.payer.toBytes(),
      authority: params.authority.toBytes(),
      emitter: params.emitter.toBytes(),
      gauge: params.gauge.toBytes(),
    }),
  )

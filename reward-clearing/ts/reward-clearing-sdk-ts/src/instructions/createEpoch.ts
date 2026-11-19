import { createEpochCreateInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createRewardClearingPda } from "../pda.ts"
import { toWeb3Instruction } from "./shared.ts"

export interface CreateEpochParams {
  payer: PublicKey
  channelGroup: PublicKey
  /** The group's current index; this creates currentEpochIndex + 1. */
  currentEpochIndex: number
}

export const createEpochInstruction = (
  programId: PublicKey,
  params: CreateEpochParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  return toWeb3Instruction(
    createEpochCreateInstruction(programId.toBytes(), {
      payer: params.payer.toBytes(),
      channelGroup: params.channelGroup.toBytes(),
      currentEpoch: pda
        .epoch(params.channelGroup, params.currentEpochIndex)
        .toBytes(),
      newEpoch: pda
        .epoch(params.channelGroup, params.currentEpochIndex + 1)
        .toBytes(),
    }),
  )
}

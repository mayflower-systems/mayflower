import { createChannelEpochFinalizeInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createRewardClearingPda } from "../pda.ts"
import { toWeb3Instruction, type ChannelEpochParams } from "./shared.ts"

export interface FinalizeChannelEpochParams extends ChannelEpochParams {
  clearingCreditsMint: PublicKey
}

/** Finalizes the explicitly selected epoch; no signer is required. */
export const finalizeChannelEpochInstruction = (
  programId: PublicKey,
  params: FinalizeChannelEpochParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  return toWeb3Instruction(
    createChannelEpochFinalizeInstruction(programId.toBytes(), {
      channel: params.channel.toBytes(),
      completedEpoch: pda
        .epoch(params.channelGroup, params.epochCounter)
        .toBytes(),
      completedChannelEpoch: pda
        .channelEpoch(params.channel, params.epochCounter)
        .toBytes(),
      clearingCreditsMint: params.clearingCreditsMint.toBytes(),
      clearingCreditsEscrowTokenAccount: pda
        .channelEpochClearingCreditsEscrow(params.channel, params.epochCounter)
        .toBytes(),
    }),
  )
}

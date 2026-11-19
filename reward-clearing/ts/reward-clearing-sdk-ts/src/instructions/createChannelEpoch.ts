import { createChannelEpochCreateInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createRewardClearingPda } from "../pda.ts"
import { toWeb3Instruction } from "./shared.ts"

export interface CreateChannelEpochParams {
  payer: PublicKey
  channelGroup: PublicKey
  /** Read from ChannelGroup.currentEpochIndex before building. */
  currentEpochIndex: number
  rewardTokenMint: PublicKey
  clearingCreditsMint: PublicKey
}

export const createChannelEpochInstruction = (
  programId: PublicKey,
  params: CreateChannelEpochParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  const channel = pda.channel(params.channelGroup, params.rewardTokenMint)
  return toWeb3Instruction(
    createChannelEpochCreateInstruction(programId.toBytes(), {
      payer: params.payer.toBytes(),
      channelGroup: params.channelGroup.toBytes(),
      channel: channel.toBytes(),
      currentEpoch: pda
        .epoch(params.channelGroup, params.currentEpochIndex)
        .toBytes(),
      newChannelEpoch: pda
        .channelEpoch(channel, params.currentEpochIndex)
        .toBytes(),
      rewardTokenMint: params.rewardTokenMint.toBytes(),
      clearingCreditsMint: params.clearingCreditsMint.toBytes(),
      rewardPotEscrowAccount: pda
        .channelEpochRewardPotEscrow(channel, params.currentEpochIndex)
        .toBytes(),
      clearingCreditsEscrowAccount: pda
        .channelEpochClearingCreditsEscrow(channel, params.currentEpochIndex)
        .toBytes(),
    }),
  )
}

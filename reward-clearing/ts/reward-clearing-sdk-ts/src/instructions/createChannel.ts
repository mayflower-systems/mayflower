import { createChannelCreateInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import type { EpochTerms } from "../models/EpochTerms.ts"
import { createRewardClearingPda } from "../pda.ts"
import { instructionEpochTerms, toWeb3Instruction } from "./shared.ts"

export interface CreateChannelParams {
  authority: PublicKey
  payer: PublicKey
  channelGroup: PublicKey
  /** Read from ChannelGroup.currentEpochIndex before building. */
  currentEpochIndex: number
  rewardTokenMint: PublicKey
  clearingCreditsMint: PublicKey
  defaultEpochTerms: EpochTerms
}

/** Creates the channel, its staging vault, and its first channel epoch. */
export const createChannelInstruction = (
  programId: PublicKey,
  params: CreateChannelParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  const channel = pda.channel(params.channelGroup, params.rewardTokenMint)
  return toWeb3Instruction(
    createChannelCreateInstruction(
      programId.toBytes(),
      {
        payer: params.payer.toBytes(),
        authority: params.authority.toBytes(),
        organization: pda.organization(params.authority).toBytes(),
        channelGroup: params.channelGroup.toBytes(),
        currentEpoch: pda
          .epoch(params.channelGroup, params.currentEpochIndex)
          .toBytes(),
        rewardTokenMint: params.rewardTokenMint.toBytes(),
        clearingCreditsMint: params.clearingCreditsMint.toBytes(),
        channel: channel.toBytes(),
        rewardStagingTokenAccount: pda
          .rewardStagingTokenAccount(channel)
          .toBytes(),
        initialChannelEpoch: pda
          .channelEpoch(channel, params.currentEpochIndex)
          .toBytes(),
        rewardPotEscrowAccount: pda
          .channelEpochRewardPotEscrow(channel, params.currentEpochIndex)
          .toBytes(),
        clearingCreditsEscrowAccount: pda
          .channelEpochClearingCreditsEscrow(channel, params.currentEpochIndex)
          .toBytes(),
      },
      {
        args: {
          defaultEpochTerms: instructionEpochTerms(params.defaultEpochTerms),
        },
      },
    ),
  )
}

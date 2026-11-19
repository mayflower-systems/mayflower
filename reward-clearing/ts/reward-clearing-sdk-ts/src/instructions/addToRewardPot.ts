import { createChannelEpochAddToRewardPotInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createRewardClearingPda } from "../pda.ts"
import {
  channelEpochAccounts,
  instructionU64,
  toWeb3Instruction,
  type ChannelEpochParams,
} from "./shared.ts"

export interface AddToRewardPotParams extends ChannelEpochParams {
  transferAuthority: PublicKey
  rewardTokenSource: PublicKey
  /** Reward token base units. */
  amount: bigint
}

export const addToRewardPotInstruction = (
  programId: PublicKey,
  params: AddToRewardPotParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  return toWeb3Instruction(
    createChannelEpochAddToRewardPotInstruction(
      programId.toBytes(),
      {
        ...channelEpochAccounts(pda, params),
        transferAuthority: params.transferAuthority.toBytes(),
        rewardPotEscrowTokenAccount: pda
          .channelEpochRewardPotEscrow(params.channel, params.epochCounter)
          .toBytes(),
        rewardTokenSrc: params.rewardTokenSource.toBytes(),
      },
      { amount: instructionU64(params.amount, "amount") },
    ),
  )
}

import { createBidderPositionClaimInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createRewardClearingPda } from "../pda.ts"
import { toWeb3Instruction } from "./shared.ts"

export interface ClaimRewardsParams {
  authority: PublicKey
  channel: PublicKey
  /** The finalized epoch being claimed, which may be an older epoch. */
  epochCounter: number
  rewardDestination: PublicKey
}

/** Claims rewards and closes the position, returning its rent to authority. */
export const claimRewardsInstruction = (
  programId: PublicKey,
  params: ClaimRewardsParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  return toWeb3Instruction(
    createBidderPositionClaimInstruction(programId.toBytes(), {
      authority: params.authority.toBytes(),
      channelEpoch: pda
        .channelEpoch(params.channel, params.epochCounter)
        .toBytes(),
      bidderPosition: pda
        .bidderPosition(params.channel, params.epochCounter, params.authority)
        .toBytes(),
      rewardPotEscrowTokenAccount: pda
        .channelEpochRewardPotEscrow(params.channel, params.epochCounter)
        .toBytes(),
      rewardDestination: params.rewardDestination.toBytes(),
    }),
  )
}

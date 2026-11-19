import { createBidderPositionCreateInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createRewardClearingPda } from "../pda.ts"
import {
  channelEpochAccounts,
  toWeb3Instruction,
  type ChannelEpochParams,
} from "./shared.ts"

export interface CreateBidderPositionParams extends ChannelEpochParams {
  authority: PublicKey
  payer: PublicKey
}

export const createBidderPositionInstruction = (
  programId: PublicKey,
  params: CreateBidderPositionParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  return toWeb3Instruction(
    createBidderPositionCreateInstruction(programId.toBytes(), {
      ...channelEpochAccounts(pda, params),
      payer: params.payer.toBytes(),
      authority: params.authority.toBytes(),
      bidderPosition: pda
        .bidderPosition(params.channel, params.epochCounter, params.authority)
        .toBytes(),
    }),
  )
}

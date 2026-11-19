import { createBidderPositionDepositInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createRewardClearingPda } from "../pda.ts"
import {
  channelEpochAccounts,
  instructionU64,
  toWeb3Instruction,
  type ChannelEpochParams,
} from "./shared.ts"

export interface DepositCreditsParams extends ChannelEpochParams {
  authority: PublicKey
  /** Funds position growth. */
  payer: PublicKey
  clearingCreditsSource: PublicKey
  /** Clearing credit base units. */
  amount: bigint
}

export const depositCreditsInstruction = (
  programId: PublicKey,
  params: DepositCreditsParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  return toWeb3Instruction(
    createBidderPositionDepositInstruction(
      programId.toBytes(),
      {
        ...channelEpochAccounts(pda, params),
        payer: params.payer.toBytes(),
        authority: params.authority.toBytes(),
        bidderPosition: pda
          .bidderPosition(params.channel, params.epochCounter, params.authority)
          .toBytes(),
        clearingCreditsEscrowTokenAccount: pda
          .channelEpochClearingCreditsEscrow(
            params.channel,
            params.epochCounter,
          )
          .toBytes(),
        clearingCreditsSource: params.clearingCreditsSource.toBytes(),
      },
      { amount: instructionU64(params.amount, "amount") },
    ),
  )
}

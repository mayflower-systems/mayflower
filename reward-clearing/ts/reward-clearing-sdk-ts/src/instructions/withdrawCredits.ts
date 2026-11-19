import { createBidderPositionWithdrawInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createRewardClearingPda } from "../pda.ts"
import {
  channelEpochAccounts,
  instructionU64,
  toWeb3Instruction,
  type ChannelEpochParams,
} from "./shared.ts"

export interface WithdrawCreditsParams extends ChannelEpochParams {
  authority: PublicKey
  clearingCreditsMint: PublicKey
  clearingCreditsDestination: PublicKey
  /** Gross clearing credits to withdraw, before the program's exit penalty. */
  amount: bigint
}

export const withdrawCreditsInstruction = (
  programId: PublicKey,
  params: WithdrawCreditsParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  return toWeb3Instruction(
    createBidderPositionWithdrawInstruction(
      programId.toBytes(),
      {
        ...channelEpochAccounts(pda, params),
        authority: params.authority.toBytes(),
        bidderPosition: pda
          .bidderPosition(params.channel, params.epochCounter, params.authority)
          .toBytes(),
        clearingCreditsMint: params.clearingCreditsMint.toBytes(),
        clearingCreditsEscrowTokenAccount: pda
          .channelEpochClearingCreditsEscrow(
            params.channel,
            params.epochCounter,
          )
          .toBytes(),
        clearingCreditsDestination: params.clearingCreditsDestination.toBytes(),
      },
      { amount: instructionU64(params.amount, "amount") },
    ),
  )
}

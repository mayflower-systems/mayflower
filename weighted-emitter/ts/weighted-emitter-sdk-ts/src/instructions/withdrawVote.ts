import { createWithdrawVoteInstruction } from "@mayflower-sys/weighted-emitter-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createWeightedEmitterPda } from "../pda.ts"
import {
  instructionU64,
  toWeb3Instruction,
  votePositionAccounts,
  type VotePositionParams,
} from "./shared.ts"

export interface WithdrawVoteParams extends VotePositionParams {
  /** Any token account for voteMint; its owner is unrestricted. */
  voteDestination: PublicKey
  /** Vote token base units. */
  amount: bigint
}

export const withdrawVoteInstruction = (
  programId: PublicKey,
  params: WithdrawVoteParams,
): TransactionInstruction => {
  const pda = createWeightedEmitterPda(programId)
  return toWeb3Instruction(
    createWithdrawVoteInstruction(
      programId.toBytes(),
      {
        ...votePositionAccounts(pda, params),
        voteDestination: params.voteDestination.toBytes(),
      },
      { amount: instructionU64(params.amount, "amount") },
    ),
  )
}

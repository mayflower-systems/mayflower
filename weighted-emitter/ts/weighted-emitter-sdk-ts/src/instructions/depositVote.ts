import { createDepositVoteInstruction } from "@mayflower-sys/weighted-emitter-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createWeightedEmitterPda } from "../pda.ts"
import {
  instructionU64,
  toWeb3Instruction,
  votePositionAccounts,
  type VotePositionParams,
} from "./shared.ts"

export interface DepositVoteParams extends VotePositionParams {
  voteSource: PublicKey
  /** Vote token base units. */
  amount: bigint
}

export const depositVoteInstruction = (
  programId: PublicKey,
  params: DepositVoteParams,
): TransactionInstruction => {
  const pda = createWeightedEmitterPda(programId)
  return toWeb3Instruction(
    createDepositVoteInstruction(
      programId.toBytes(),
      {
        ...votePositionAccounts(pda, params),
        voteSource: params.voteSource.toBytes(),
      },
      { amount: instructionU64(params.amount, "amount") },
    ),
  )
}

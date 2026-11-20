import { createCreateVotePositionInstruction } from "@mayflower-sys/weighted-emitter-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createWeightedEmitterPda } from "../pda.ts"
import {
  toWeb3Instruction,
  votePositionAccounts,
  type VotePositionParams,
} from "./shared.ts"

export interface CreateVotePositionParams extends VotePositionParams {
  payer: PublicKey
}

/** Creates the authority's position and its token escrow for this gauge. */
export const createVotePositionInstruction = (
  programId: PublicKey,
  params: CreateVotePositionParams,
): TransactionInstruction => {
  const pda = createWeightedEmitterPda(programId)
  return toWeb3Instruction(
    createCreateVotePositionInstruction(programId.toBytes(), {
      ...votePositionAccounts(pda, params),
      payer: params.payer.toBytes(),
    }),
  )
}

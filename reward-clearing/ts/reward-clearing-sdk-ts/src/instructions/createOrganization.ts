import { createCreateOrganizationInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createRewardClearingPda } from "../pda.ts"
import { toWeb3Instruction } from "./shared.ts"

export interface CreateOrganizationParams {
  authority: PublicKey
  payer: PublicKey
}

export const createOrganizationInstruction = (
  programId: PublicKey,
  params: CreateOrganizationParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  return toWeb3Instruction(
    createCreateOrganizationInstruction(programId.toBytes(), {
      payer: params.payer.toBytes(),
      authority: params.authority.toBytes(),
      organization: pda.organization(params.authority).toBytes(),
    }),
  )
}

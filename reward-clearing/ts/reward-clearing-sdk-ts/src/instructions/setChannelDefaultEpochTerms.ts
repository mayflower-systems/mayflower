import { createChannelSetDefaultEpochTermsInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import type { EpochTerms } from "../models/EpochTerms.ts"
import { createRewardClearingPda } from "../pda.ts"
import { instructionEpochTerms, toWeb3Instruction } from "./shared.ts"

export interface SetChannelDefaultEpochTermsParams {
  authority: PublicKey
  channelGroup: PublicKey
  channel: PublicKey
  defaultEpochTerms: EpochTerms
}

export const setChannelDefaultEpochTermsInstruction = (
  programId: PublicKey,
  params: SetChannelDefaultEpochTermsParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  return toWeb3Instruction(
    createChannelSetDefaultEpochTermsInstruction(
      programId.toBytes(),
      {
        authority: params.authority.toBytes(),
        organization: pda.organization(params.authority).toBytes(),
        channelGroup: params.channelGroup.toBytes(),
        channel: params.channel.toBytes(),
      },
      {
        args: {
          defaultEpochTerms: instructionEpochTerms(params.defaultEpochTerms),
        },
      },
    ),
  )
}

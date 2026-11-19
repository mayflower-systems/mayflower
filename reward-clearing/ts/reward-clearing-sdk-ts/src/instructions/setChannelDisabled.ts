import { createChannelSetDisabledInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createRewardClearingPda } from "../pda.ts"
import { toWeb3Instruction } from "./shared.ts"

export interface SetChannelDisabledParams {
  authority: PublicKey
  channelGroup: PublicKey
  channel: PublicKey
  disabled: boolean
}

export const setChannelDisabledInstruction = (
  programId: PublicKey,
  params: SetChannelDisabledParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  return toWeb3Instruction(
    createChannelSetDisabledInstruction(
      programId.toBytes(),
      {
        authority: params.authority.toBytes(),
        organization: pda.organization(params.authority).toBytes(),
        channelGroup: params.channelGroup.toBytes(),
        channel: params.channel.toBytes(),
      },
      { args: { disabled: params.disabled } },
    ),
  )
}

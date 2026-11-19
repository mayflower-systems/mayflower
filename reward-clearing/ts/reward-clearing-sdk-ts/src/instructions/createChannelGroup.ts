import { createCreateChannelGroupInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import type { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { createRewardClearingPda } from "../pda.ts"
import { instructionU32, instructionU64, toWeb3Instruction } from "./shared.ts"

export interface CreateChannelGroupParams {
  authority: PublicKey
  payer: PublicKey
  /** Read from Organization.nextChannelGroupIndex before building. */
  nextChannelGroupIndex: number
  clearingCreditsMint: PublicKey
  masterStartTimeUnixSeconds: bigint
  epochDurationSeconds: number
}

/** Creates the numbered group, epoch 0, and the group's clearing escrow. */
export const createChannelGroupInstruction = (
  programId: PublicKey,
  params: CreateChannelGroupParams,
): TransactionInstruction => {
  const pda = createRewardClearingPda(programId)
  const organization = pda.organization(params.authority)
  const channelGroup = pda.channelGroup(
    organization,
    params.nextChannelGroupIndex,
  )
  return toWeb3Instruction(
    createCreateChannelGroupInstruction(
      programId.toBytes(),
      {
        payer: params.payer.toBytes(),
        authority: params.authority.toBytes(),
        organization: organization.toBytes(),
        channelGroup: channelGroup.toBytes(),
        initialEpoch: pda.epoch(channelGroup, 0).toBytes(),
        clearingCreditsMint: params.clearingCreditsMint.toBytes(),
        clearingCreditsEscrow: pda.clearingTokenEscrow(channelGroup).toBytes(),
      },
      {
        args: {
          masterStartTimeUnixSeconds: instructionU64(
            params.masterStartTimeUnixSeconds,
            "masterStartTimeUnixSeconds",
          ),
          epochDurationSeconds: instructionU32(
            params.epochDurationSeconds,
            "epochDurationSeconds",
          ),
        },
      },
    ),
  )
}

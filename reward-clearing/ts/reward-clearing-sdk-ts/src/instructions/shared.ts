import type { TransactionInstruction as ClientInstruction } from "@mayflower-sys/reward-clearing-sol-client"
import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { Buffer } from "buffer"
import * as EpochTerms from "../models/EpochTerms.ts"
import type { RewardClearingPda } from "../pda.ts"

/** Identifies an epoch of an existing channel without reading account state. */
export interface ChannelEpochParams {
  channelGroup: PublicKey
  channel: PublicKey
  epochCounter: number
}

export const channelEpochAccounts = (
  pda: RewardClearingPda,
  params: ChannelEpochParams,
) => ({
  channelGroup: params.channelGroup.toBytes(),
  channel: params.channel.toBytes(),
  epoch: pda.epoch(params.channelGroup, params.epochCounter).toBytes(),
  channelEpoch: pda.channelEpoch(params.channel, params.epochCounter).toBytes(),
})

const unsignedInteger = (
  value: number,
  maximum: number,
  name: string,
): number => {
  if (!Number.isInteger(value) || value < 0 || value > maximum) {
    throw new RangeError(`${name} must be an integer from 0 to ${maximum}`)
  }
  return value
}

// Borsh's encoder truncates integers that do not fit. Check their storage range
// before encoding; business constraints remain the program's responsibility.
export const instructionU32 = (value: number, name: string): number =>
  unsignedInteger(value, 0xffff_ffff, name)

export const instructionU64 = (value: bigint, name: string): bigint => {
  if (value < 0n || value > 0xffff_ffff_ffff_ffffn) {
    throw new RangeError(
      `${name} must be an integer from 0 to 18446744073709551615`,
    )
  }
  return value
}

export const instructionEpochTerms = (terms: EpochTerms.EpochTerms) => {
  const result = EpochTerms.toAccount(terms)
  if (!result.ok) {
    throw new RangeError(`defaultEpochTerms.rampBonusSlope: ${result.error}`)
  }
  return {
    ...result.value,
    exitPenaltyExponent: unsignedInteger(
      result.value.exitPenaltyExponent,
      0xff,
      "defaultEpochTerms.exitPenaltyExponent",
    ),
  }
}

/** Keeps the generated client's encoding and account access flags intact. */
export const toWeb3Instruction = (
  instruction: ClientInstruction,
): TransactionInstruction =>
  new TransactionInstruction({
    programId: new PublicKey(instruction.programId),
    keys: instruction.keys.map(({ pubkey, isSigner, isWritable }) => ({
      pubkey: new PublicKey(pubkey),
      isSigner,
      isWritable,
    })),
    data: Buffer.from(instruction.data),
  })

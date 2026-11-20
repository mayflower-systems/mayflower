import type { TransactionInstruction as ClientInstruction } from "@mayflower-sys/weighted-emitter-sol-client"
import { PublicKey, TransactionInstruction } from "@solana/web3.js"
import { Buffer } from "buffer"
import * as BigDecimal from "../models/BigDecimal.ts"
import type { WeightedEmitterPda } from "../pda.ts"

/** The explicit accounts needed to identify a voting position without RPC. */
export interface VotePositionParams {
  authority: PublicKey
  emitter: PublicKey
  gauge: PublicKey
  voteMint: PublicKey
}

export const votePositionAccounts = (
  pda: WeightedEmitterPda,
  params: VotePositionParams,
) => {
  const votePosition = pda.votePosition(params.gauge, params.authority)
  return {
    authority: params.authority.toBytes(),
    emitter: params.emitter.toBytes(),
    gauge: params.gauge.toBytes(),
    voteMint: params.voteMint.toBytes(),
    votePosition: votePosition.toBytes(),
    voteEscrow: pda.voteEscrow(votePosition).toBytes(),
  }
}

/** Borsh truncates out-of-range integers, so check the storage range first. */
export const instructionU64 = (value: bigint, name: string): bigint => {
  if (value < 0n || value > 0xffff_ffff_ffff_ffffn) {
    throw new RangeError(
      `${name} must be an integer from 0 to 18446744073709551615`,
    )
  }
  return value
}

/** Accepts only rates that the program's u128 Q64.64 format holds exactly. */
export const instructionRate = (
  value: BigDecimal.BigDecimal,
  name: string,
): bigint => {
  const result = BigDecimal.toQ64f64(value)
  if (!result.ok) throw new RangeError(`${name}: ${result.error}`)
  return result.value
}

/** The generated client owns encoding, account order, and access flags. */
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

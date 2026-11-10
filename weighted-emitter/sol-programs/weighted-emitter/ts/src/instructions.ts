// Generated from an Anchor IDL. Do not edit.

import type { Schema } from "borsh"
import { encodeData } from "./codecs.js"
import * as models from "./models.js"
import {
  publicKeyFromBase58,
  type AccountMeta,
  type PublicKey,
  type TransactionInstruction,
} from "./solana.js"

// fixed addresses ---------------------------------------------------

export const SYSTEM_PROGRAM_ADDRESS = "11111111111111111111111111111111"
export const SYSTEM_PROGRAM_ID: PublicKey = publicKeyFromBase58(
  SYSTEM_PROGRAM_ADDRESS,
)
export const TOKEN_PROGRAM_ADDRESS =
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
export const TOKEN_PROGRAM_ID: PublicKey = publicKeyFromBase58(
  TOKEN_PROGRAM_ADDRESS,
)

// instruction schemas -----------------------------------------------

export const CREATE_EMITTER_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  119, 238, 254, 153, 168, 36, 22, 201,
])
export interface CreateEmitterInstructionArgs {
  args: models.CreateEmitterArgs
}
export const CREATE_EMITTER_INSTRUCTION_SCHEMA: Schema = {
  struct: { args: models.CREATE_EMITTER_ARGS_SCHEMA },
}

export const CREATE_GAUGE_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  36, 92, 148, 102, 185, 224, 141, 126,
])
export type CreateGaugeInstructionArgs = Record<string, never>
export const CREATE_GAUGE_INSTRUCTION_SCHEMA: Schema = { struct: {} }

export const CREATE_VOTE_POSITION_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  209, 143, 132, 123, 199, 109, 170, 166,
])
export type CreateVotePositionInstructionArgs = Record<string, never>
export const CREATE_VOTE_POSITION_INSTRUCTION_SCHEMA: Schema = { struct: {} }

export const DEPOSIT_VOTE_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  242, 79, 103, 5, 170, 216, 43, 103,
])
export interface DepositVoteInstructionArgs {
  amount: bigint
}
export const DEPOSIT_VOTE_INSTRUCTION_SCHEMA: Schema = {
  struct: { amount: "u64" },
}

export const SET_EMITTER_DECAY_RATE_INSTRUCTION_DISCRIMINATOR = Uint8Array.from(
  [55, 248, 227, 148, 156, 232, 95, 53],
)
export interface SetEmitterDecayRateInstructionArgs {
  args: models.SetEmitterDecayRateArgs
}
export const SET_EMITTER_DECAY_RATE_INSTRUCTION_SCHEMA: Schema = {
  struct: { args: models.SET_EMITTER_DECAY_RATE_ARGS_SCHEMA },
}

export const SETTLE_EMISSION_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  206, 121, 221, 190, 111, 232, 51, 13,
])
export type SettleEmissionInstructionArgs = Record<string, never>
export const SETTLE_EMISSION_INSTRUCTION_SCHEMA: Schema = { struct: {} }

export const WITHDRAW_VOTE_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  243, 255, 70, 200, 3, 242, 103, 137,
])
export interface WithdrawVoteInstructionArgs {
  amount: bigint
}
export const WITHDRAW_VOTE_INSTRUCTION_SCHEMA: Schema = {
  struct: { amount: "u64" },
}

// instruction builders ----------------------------------------------

export interface CreateEmitterAccounts {
  payer: PublicKey
  authority: PublicKey
  emitter: PublicKey
  voteMint: PublicKey
  emissionMint: PublicKey
  mintAuthority: PublicKey
}

/**
 * Builds account metas for `create_emitter` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createCreateEmitterAccountMetas = (
  accounts: CreateEmitterAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.emitter, isWritable: true, isSigner: true },
    { pubkey: accounts.voteMint, isWritable: false, isSigner: false },
    { pubkey: accounts.emissionMint, isWritable: false, isSigner: false },
    { pubkey: accounts.mintAuthority, isWritable: false, isSigner: false },
    { pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createCreateEmitterInstruction = (
  programId: PublicKey,
  accounts: CreateEmitterAccounts,
  args: CreateEmitterInstructionArgs,
): TransactionInstruction => {
  const keys = createCreateEmitterAccountMetas(accounts)
  const data = encodeData(
    CREATE_EMITTER_INSTRUCTION_SCHEMA,
    CREATE_EMITTER_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return { programId, keys, data }
}

export interface CreateGaugeAccounts {
  payer: PublicKey
  authority: PublicKey
  emitter: PublicKey
  gauge: PublicKey
}

/**
 * Builds account metas for `create_gauge` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createCreateGaugeAccountMetas = (
  accounts: CreateGaugeAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.emitter, isWritable: true, isSigner: false },
    { pubkey: accounts.gauge, isWritable: true, isSigner: true },
    { pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createCreateGaugeInstruction = (
  programId: PublicKey,
  accounts: CreateGaugeAccounts,
): TransactionInstruction => {
  const keys = createCreateGaugeAccountMetas(accounts)
  const data = encodeData(
    CREATE_GAUGE_INSTRUCTION_SCHEMA,
    CREATE_GAUGE_INSTRUCTION_DISCRIMINATOR,
    {},
  )
  return { programId, keys, data }
}

export interface CreateVotePositionAccounts {
  payer: PublicKey
  authority: PublicKey
  emitter: PublicKey
  gauge: PublicKey
  votePosition: PublicKey
  voteMint: PublicKey
  voteEscrow: PublicKey
}

/**
 * Builds account metas for `create_vote_position` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createCreateVotePositionAccountMetas = (
  accounts: CreateVotePositionAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.emitter, isWritable: false, isSigner: false },
    { pubkey: accounts.gauge, isWritable: false, isSigner: false },
    { pubkey: accounts.votePosition, isWritable: true, isSigner: false },
    { pubkey: accounts.voteMint, isWritable: false, isSigner: false },
    { pubkey: accounts.voteEscrow, isWritable: true, isSigner: false },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createCreateVotePositionInstruction = (
  programId: PublicKey,
  accounts: CreateVotePositionAccounts,
): TransactionInstruction => {
  const keys = createCreateVotePositionAccountMetas(accounts)
  const data = encodeData(
    CREATE_VOTE_POSITION_INSTRUCTION_SCHEMA,
    CREATE_VOTE_POSITION_INSTRUCTION_DISCRIMINATOR,
    {},
  )
  return { programId, keys, data }
}

export interface DepositVoteAccounts {
  authority: PublicKey
  emitter: PublicKey
  gauge: PublicKey
  votePosition: PublicKey
  voteMint: PublicKey
  voteEscrow: PublicKey
  voteSource: PublicKey
}

/**
 * Builds account metas for `deposit_vote` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createDepositVoteAccountMetas = (
  accounts: DepositVoteAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.emitter, isWritable: true, isSigner: false },
    { pubkey: accounts.gauge, isWritable: true, isSigner: false },
    { pubkey: accounts.votePosition, isWritable: true, isSigner: false },
    { pubkey: accounts.voteMint, isWritable: false, isSigner: false },
    { pubkey: accounts.voteEscrow, isWritable: true, isSigner: false },
    { pubkey: accounts.voteSource, isWritable: true, isSigner: false },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createDepositVoteInstruction = (
  programId: PublicKey,
  accounts: DepositVoteAccounts,
  args: DepositVoteInstructionArgs,
): TransactionInstruction => {
  const keys = createDepositVoteAccountMetas(accounts)
  const data = encodeData(
    DEPOSIT_VOTE_INSTRUCTION_SCHEMA,
    DEPOSIT_VOTE_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return { programId, keys, data }
}

export interface SetEmitterDecayRateAccounts {
  authority: PublicKey
  emitter: PublicKey
}

/**
 * Builds account metas for `set_emitter_decay_rate` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createSetEmitterDecayRateAccountMetas = (
  accounts: SetEmitterDecayRateAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.emitter, isWritable: true, isSigner: false },
  ]
}

export const createSetEmitterDecayRateInstruction = (
  programId: PublicKey,
  accounts: SetEmitterDecayRateAccounts,
  args: SetEmitterDecayRateInstructionArgs,
): TransactionInstruction => {
  const keys = createSetEmitterDecayRateAccountMetas(accounts)
  const data = encodeData(
    SET_EMITTER_DECAY_RATE_INSTRUCTION_SCHEMA,
    SET_EMITTER_DECAY_RATE_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return { programId, keys, data }
}

export interface SettleEmissionAccounts {
  authority: PublicKey
  emitter: PublicKey
  gauge: PublicKey
  emissionMint: PublicKey
  mintAuthority: PublicKey
  /**
   * The emitter authority chooses the recipient; its token owner is unrestricted.
   */
  emissionDestination: PublicKey
}

/**
 * Builds account metas for `settle_emission` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createSettleEmissionAccountMetas = (
  accounts: SettleEmissionAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.emitter, isWritable: true, isSigner: false },
    { pubkey: accounts.gauge, isWritable: true, isSigner: false },
    { pubkey: accounts.emissionMint, isWritable: true, isSigner: false },
    { pubkey: accounts.mintAuthority, isWritable: false, isSigner: false },
    { pubkey: accounts.emissionDestination, isWritable: true, isSigner: false },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createSettleEmissionInstruction = (
  programId: PublicKey,
  accounts: SettleEmissionAccounts,
): TransactionInstruction => {
  const keys = createSettleEmissionAccountMetas(accounts)
  const data = encodeData(
    SETTLE_EMISSION_INSTRUCTION_SCHEMA,
    SETTLE_EMISSION_INSTRUCTION_DISCRIMINATOR,
    {},
  )
  return { programId, keys, data }
}

export interface WithdrawVoteAccounts {
  authority: PublicKey
  emitter: PublicKey
  gauge: PublicKey
  votePosition: PublicKey
  voteMint: PublicKey
  voteEscrow: PublicKey
  voteDestination: PublicKey
}

/**
 * Builds account metas for `withdraw_vote` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createWithdrawVoteAccountMetas = (
  accounts: WithdrawVoteAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.emitter, isWritable: true, isSigner: false },
    { pubkey: accounts.gauge, isWritable: true, isSigner: false },
    { pubkey: accounts.votePosition, isWritable: true, isSigner: false },
    { pubkey: accounts.voteMint, isWritable: false, isSigner: false },
    { pubkey: accounts.voteEscrow, isWritable: true, isSigner: false },
    { pubkey: accounts.voteDestination, isWritable: true, isSigner: false },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createWithdrawVoteInstruction = (
  programId: PublicKey,
  accounts: WithdrawVoteAccounts,
  args: WithdrawVoteInstructionArgs,
): TransactionInstruction => {
  const keys = createWithdrawVoteAccountMetas(accounts)
  const data = encodeData(
    WITHDRAW_VOTE_INSTRUCTION_SCHEMA,
    WITHDRAW_VOTE_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return { programId, keys, data }
}

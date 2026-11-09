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

// instruction schemas -----------------------------------------------

export const CALCULATE_SPANS_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  153, 97, 174, 93, 213, 190, 207, 24,
])
export interface CalculateSpansInstructionArgs {
  args: models.CalculateSpansArgs
}
export const CALCULATE_SPANS_INSTRUCTION_SCHEMA: Schema = {
  struct: { args: models.CALCULATE_SPANS_ARGS_SCHEMA },
}

export const CREATE_LINEAR_ENGINE_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  203, 156, 46, 120, 92, 192, 163, 220,
])
export interface CreateLinearEngineInstructionArgs {
  args: models.CreateLinearEngineArgs
}
export const CREATE_LINEAR_ENGINE_INSTRUCTION_SCHEMA: Schema = {
  struct: { args: models.CREATE_LINEAR_ENGINE_ARGS_SCHEMA },
}

// instruction builders ----------------------------------------------

export interface CalculateSpansAccounts {
  linearCharacteristicParam: PublicKey
}

/**
 * Builds account metas for `calculate_spans` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createCalculateSpansAccountMetas = (
  accounts: CalculateSpansAccounts,
): AccountMeta[] => {
  return [
    {
      pubkey: accounts.linearCharacteristicParam,
      isWritable: false,
      isSigner: false,
    },
  ]
}

/**
 * Evaluate neutral characteristic spans and return their bounds in order.
 */
export const createCalculateSpansInstruction = (
  programId: PublicKey,
  accounts: CalculateSpansAccounts,
  args: CalculateSpansInstructionArgs,
): TransactionInstruction => {
  const keys = createCalculateSpansAccountMetas(accounts)
  const data = encodeData(
    CALCULATE_SPANS_INSTRUCTION_SCHEMA,
    CALCULATE_SPANS_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return { programId, keys, data }
}

export interface CreateLinearEngineAccounts {
  payer: PublicKey
  linearCharacteristicParam: PublicKey
}

/**
 * Builds account metas for `create_linear_engine` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createCreateLinearEngineAccountMetas = (
  accounts: CreateLinearEngineAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    {
      pubkey: accounts.linearCharacteristicParam,
      isWritable: true,
      isSigner: true,
    },
    { pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createCreateLinearEngineInstruction = (
  programId: PublicKey,
  accounts: CreateLinearEngineAccounts,
  args: CreateLinearEngineInstructionArgs,
): TransactionInstruction => {
  const keys = createCreateLinearEngineAccountMetas(accounts)
  const data = encodeData(
    CREATE_LINEAR_ENGINE_INSTRUCTION_SCHEMA,
    CREATE_LINEAR_ENGINE_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return { programId, keys, data }
}

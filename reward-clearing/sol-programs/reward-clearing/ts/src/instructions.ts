// Generated from an Anchor IDL. Do not edit.

import type { Schema } from "borsh"
import bs58 from "bs58"
import {
  PublicKey,
  TransactionInstruction,
  type AccountMeta,
} from "@solana/web3.js"
import { encodeData } from "./codecs.js"
import * as models from "./models.js"

// fixed addresses ---------------------------------------------------

export const TOKEN_PROGRAM_ADDRESS = bs58.encode(
  Uint8Array.from([
    6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172,
    28, 180, 133, 237, 95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169,
  ]),
)
export const TOKEN_PROGRAM_ID = new PublicKey(TOKEN_PROGRAM_ADDRESS)
export const SYSTEM_PROGRAM_ADDRESS = bs58.encode(
  Uint8Array.from([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0,
  ]),
)
export const SYSTEM_PROGRAM_ID = new PublicKey(SYSTEM_PROGRAM_ADDRESS)

// instruction schemas -----------------------------------------------

export const BIDDER_POSITION_CLAIM_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  216, 136, 87, 58, 21, 37, 116, 249,
])
export type BidderPositionClaimInstructionArgs = Record<string, never>
export const BIDDER_POSITION_CLAIM_INSTRUCTION_SCHEMA: Schema = { struct: {} }

export const BIDDER_POSITION_CREATE_INSTRUCTION_DISCRIMINATOR = Uint8Array.from(
  [225, 232, 121, 179, 79, 235, 118, 122],
)
export type BidderPositionCreateInstructionArgs = Record<string, never>
export const BIDDER_POSITION_CREATE_INSTRUCTION_SCHEMA: Schema = { struct: {} }

export const BIDDER_POSITION_DEPOSIT_INSTRUCTION_DISCRIMINATOR =
  Uint8Array.from([163, 192, 90, 123, 195, 45, 120, 164])
export interface BidderPositionDepositInstructionArgs {
  amount: bigint
}
export const BIDDER_POSITION_DEPOSIT_INSTRUCTION_SCHEMA: Schema = {
  struct: { amount: "u64" },
}

export const BIDDER_POSITION_WITHDRAW_INSTRUCTION_DISCRIMINATOR =
  Uint8Array.from([186, 242, 235, 128, 119, 131, 157, 210])
export interface BidderPositionWithdrawInstructionArgs {
  amount: bigint
}
export const BIDDER_POSITION_WITHDRAW_INSTRUCTION_SCHEMA: Schema = {
  struct: { amount: "u64" },
}

export const CHANNEL_CREATE_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  213, 253, 72, 169, 211, 191, 116, 189,
])
export interface ChannelCreateInstructionArgs {
  args: models.ChannelCreateArgs
}
export const CHANNEL_CREATE_INSTRUCTION_SCHEMA: Schema = {
  struct: { args: models.CHANNEL_CREATE_ARGS_SCHEMA },
}

export const CHANNEL_EPOCH_ADD_TO_REWARD_POT_INSTRUCTION_DISCRIMINATOR =
  Uint8Array.from([128, 16, 205, 229, 125, 214, 9, 11])
export interface ChannelEpochAddToRewardPotInstructionArgs {
  amount: bigint
}
export const CHANNEL_EPOCH_ADD_TO_REWARD_POT_INSTRUCTION_SCHEMA: Schema = {
  struct: { amount: "u64" },
}

export const CHANNEL_EPOCH_CREATE_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  154, 192, 198, 231, 244, 192, 167, 20,
])
export type ChannelEpochCreateInstructionArgs = Record<string, never>
export const CHANNEL_EPOCH_CREATE_INSTRUCTION_SCHEMA: Schema = { struct: {} }

export const CHANNEL_EPOCH_FINALIZE_INSTRUCTION_DISCRIMINATOR = Uint8Array.from(
  [129, 26, 243, 5, 250, 63, 83, 1],
)
export type ChannelEpochFinalizeInstructionArgs = Record<string, never>
export const CHANNEL_EPOCH_FINALIZE_INSTRUCTION_SCHEMA: Schema = { struct: {} }

export const CHANNEL_SET_DEFAULT_EPOCH_TERMS_INSTRUCTION_DISCRIMINATOR =
  Uint8Array.from([176, 169, 203, 1, 170, 74, 8, 150])
export interface ChannelSetDefaultEpochTermsInstructionArgs {
  args: models.ChannelSetDefaultEpochTermsArgs
}
export const CHANNEL_SET_DEFAULT_EPOCH_TERMS_INSTRUCTION_SCHEMA: Schema = {
  struct: { args: models.CHANNEL_SET_DEFAULT_EPOCH_TERMS_ARGS_SCHEMA },
}

export const CHANNEL_SET_DISABLED_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  84, 110, 177, 66, 32, 78, 65, 177,
])
export interface ChannelSetDisabledInstructionArgs {
  args: models.ChannelSetDisabledArgs
}
export const CHANNEL_SET_DISABLED_INSTRUCTION_SCHEMA: Schema = {
  struct: { args: models.CHANNEL_SET_DISABLED_ARGS_SCHEMA },
}

export const CREATE_CHANNEL_GROUP_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  58, 250, 243, 142, 233, 80, 83, 75,
])
export interface CreateChannelGroupInstructionArgs {
  args: models.CreateChannelGroupArgs
}
export const CREATE_CHANNEL_GROUP_INSTRUCTION_SCHEMA: Schema = {
  struct: { args: models.CREATE_CHANNEL_GROUP_ARGS_SCHEMA },
}

export const CREATE_ORGANIZATION_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  60, 173, 177, 39, 122, 23, 68, 185,
])
export type CreateOrganizationInstructionArgs = Record<string, never>
export const CREATE_ORGANIZATION_INSTRUCTION_SCHEMA: Schema = { struct: {} }

export const EPOCH_CREATE_INSTRUCTION_DISCRIMINATOR = Uint8Array.from([
  242, 185, 137, 95, 234, 33, 208, 126,
])
export type EpochCreateInstructionArgs = Record<string, never>
export const EPOCH_CREATE_INSTRUCTION_SCHEMA: Schema = { struct: {} }

// instruction builders ----------------------------------------------

export interface BidderPositionClaimAccounts {
  authority: PublicKey
  channelEpoch: PublicKey
  bidderPosition: PublicKey
  rewardPotEscrowTokenAccount: PublicKey
  rewardDestination: PublicKey
}

/**
 * Builds account metas for `bidder_position_claim` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createBidderPositionClaimAccountMetas = (
  accounts: BidderPositionClaimAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.authority, isWritable: true, isSigner: true },
    { pubkey: accounts.channelEpoch, isWritable: true, isSigner: false },
    { pubkey: accounts.bidderPosition, isWritable: true, isSigner: false },
    {
      pubkey: accounts.rewardPotEscrowTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: accounts.rewardDestination, isWritable: true, isSigner: false },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createBidderPositionClaimInstruction = (
  programId: PublicKey,
  accounts: BidderPositionClaimAccounts,
): TransactionInstruction => {
  const keys = createBidderPositionClaimAccountMetas(accounts)
  const data = encodeData(
    BIDDER_POSITION_CLAIM_INSTRUCTION_SCHEMA,
    BIDDER_POSITION_CLAIM_INSTRUCTION_DISCRIMINATOR,
    {},
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface BidderPositionCreateAccounts {
  payer: PublicKey
  authority: PublicKey
  channelGroup: PublicKey
  channel: PublicKey
  epoch: PublicKey
  channelEpoch: PublicKey
  bidderPosition: PublicKey
}

/**
 * Builds account metas for `bidder_position_create` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createBidderPositionCreateAccountMetas = (
  accounts: BidderPositionCreateAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.channelGroup, isWritable: false, isSigner: false },
    { pubkey: accounts.channel, isWritable: false, isSigner: false },
    { pubkey: accounts.epoch, isWritable: false, isSigner: false },
    { pubkey: accounts.channelEpoch, isWritable: false, isSigner: false },
    { pubkey: accounts.bidderPosition, isWritable: true, isSigner: false },
    { pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createBidderPositionCreateInstruction = (
  programId: PublicKey,
  accounts: BidderPositionCreateAccounts,
): TransactionInstruction => {
  const keys = createBidderPositionCreateAccountMetas(accounts)
  const data = encodeData(
    BIDDER_POSITION_CREATE_INSTRUCTION_SCHEMA,
    BIDDER_POSITION_CREATE_INSTRUCTION_DISCRIMINATOR,
    {},
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface BidderPositionDepositAccounts {
  payer: PublicKey
  authority: PublicKey
  channelGroup: PublicKey
  channel: PublicKey
  epoch: PublicKey
  channelEpoch: PublicKey
  bidderPosition: PublicKey
  clearingCreditsEscrowTokenAccount: PublicKey
  clearingCreditsSource: PublicKey
}

/**
 * Builds account metas for `bidder_position_deposit` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createBidderPositionDepositAccountMetas = (
  accounts: BidderPositionDepositAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.channelGroup, isWritable: false, isSigner: false },
    { pubkey: accounts.channel, isWritable: false, isSigner: false },
    { pubkey: accounts.epoch, isWritable: false, isSigner: false },
    { pubkey: accounts.channelEpoch, isWritable: true, isSigner: false },
    { pubkey: accounts.bidderPosition, isWritable: true, isSigner: false },
    {
      pubkey: accounts.clearingCreditsEscrowTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.clearingCreditsSource,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createBidderPositionDepositInstruction = (
  programId: PublicKey,
  accounts: BidderPositionDepositAccounts,
  args: BidderPositionDepositInstructionArgs,
): TransactionInstruction => {
  const keys = createBidderPositionDepositAccountMetas(accounts)
  const data = encodeData(
    BIDDER_POSITION_DEPOSIT_INSTRUCTION_SCHEMA,
    BIDDER_POSITION_DEPOSIT_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface BidderPositionWithdrawAccounts {
  authority: PublicKey
  channelGroup: PublicKey
  channel: PublicKey
  epoch: PublicKey
  channelEpoch: PublicKey
  bidderPosition: PublicKey
  clearingCreditsMint: PublicKey
  clearingCreditsEscrowTokenAccount: PublicKey
  clearingCreditsDestination: PublicKey
}

/**
 * Builds account metas for `bidder_position_withdraw` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createBidderPositionWithdrawAccountMetas = (
  accounts: BidderPositionWithdrawAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.channelGroup, isWritable: false, isSigner: false },
    { pubkey: accounts.channel, isWritable: false, isSigner: false },
    { pubkey: accounts.epoch, isWritable: false, isSigner: false },
    { pubkey: accounts.channelEpoch, isWritable: true, isSigner: false },
    { pubkey: accounts.bidderPosition, isWritable: true, isSigner: false },
    { pubkey: accounts.clearingCreditsMint, isWritable: true, isSigner: false },
    {
      pubkey: accounts.clearingCreditsEscrowTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.clearingCreditsDestination,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createBidderPositionWithdrawInstruction = (
  programId: PublicKey,
  accounts: BidderPositionWithdrawAccounts,
  args: BidderPositionWithdrawInstructionArgs,
): TransactionInstruction => {
  const keys = createBidderPositionWithdrawAccountMetas(accounts)
  const data = encodeData(
    BIDDER_POSITION_WITHDRAW_INSTRUCTION_SCHEMA,
    BIDDER_POSITION_WITHDRAW_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface ChannelCreateAccounts {
  payer: PublicKey
  authority: PublicKey
  organization: PublicKey
  channelGroup: PublicKey
  currentEpoch: PublicKey
  rewardTokenMint: PublicKey
  clearingCreditsMint: PublicKey
  channel: PublicKey
  rewardStagingTokenAccount: PublicKey
  initialChannelEpoch: PublicKey
  rewardPotEscrowAccount: PublicKey
  clearingCreditsEscrowAccount: PublicKey
}

/**
 * Builds account metas for `channel_create` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createChannelCreateAccountMetas = (
  accounts: ChannelCreateAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.organization, isWritable: false, isSigner: false },
    { pubkey: accounts.channelGroup, isWritable: false, isSigner: false },
    { pubkey: accounts.currentEpoch, isWritable: false, isSigner: false },
    { pubkey: accounts.rewardTokenMint, isWritable: false, isSigner: false },
    {
      pubkey: accounts.clearingCreditsMint,
      isWritable: false,
      isSigner: false,
    },
    { pubkey: accounts.channel, isWritable: true, isSigner: false },
    {
      pubkey: accounts.rewardStagingTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: accounts.initialChannelEpoch, isWritable: true, isSigner: false },
    {
      pubkey: accounts.rewardPotEscrowAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.clearingCreditsEscrowAccount,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createChannelCreateInstruction = (
  programId: PublicKey,
  accounts: ChannelCreateAccounts,
  args: ChannelCreateInstructionArgs,
): TransactionInstruction => {
  const keys = createChannelCreateAccountMetas(accounts)
  const data = encodeData(
    CHANNEL_CREATE_INSTRUCTION_SCHEMA,
    CHANNEL_CREATE_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface ChannelEpochAddToRewardPotAccounts {
  transferAuthority: PublicKey
  channelGroup: PublicKey
  channel: PublicKey
  epoch: PublicKey
  channelEpoch: PublicKey
  rewardPotEscrowTokenAccount: PublicKey
  rewardTokenSrc: PublicKey
}

/**
 * Builds account metas for `channel_epoch_add_to_reward_pot` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createChannelEpochAddToRewardPotAccountMetas = (
  accounts: ChannelEpochAddToRewardPotAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.transferAuthority, isWritable: false, isSigner: true },
    { pubkey: accounts.channelGroup, isWritable: false, isSigner: false },
    { pubkey: accounts.channel, isWritable: false, isSigner: false },
    { pubkey: accounts.epoch, isWritable: false, isSigner: false },
    { pubkey: accounts.channelEpoch, isWritable: true, isSigner: false },
    {
      pubkey: accounts.rewardPotEscrowTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: accounts.rewardTokenSrc, isWritable: true, isSigner: false },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createChannelEpochAddToRewardPotInstruction = (
  programId: PublicKey,
  accounts: ChannelEpochAddToRewardPotAccounts,
  args: ChannelEpochAddToRewardPotInstructionArgs,
): TransactionInstruction => {
  const keys = createChannelEpochAddToRewardPotAccountMetas(accounts)
  const data = encodeData(
    CHANNEL_EPOCH_ADD_TO_REWARD_POT_INSTRUCTION_SCHEMA,
    CHANNEL_EPOCH_ADD_TO_REWARD_POT_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface ChannelEpochCreateAccounts {
  payer: PublicKey
  channelGroup: PublicKey
  channel: PublicKey
  currentEpoch: PublicKey
  newChannelEpoch: PublicKey
  rewardTokenMint: PublicKey
  clearingCreditsMint: PublicKey
  rewardPotEscrowAccount: PublicKey
  clearingCreditsEscrowAccount: PublicKey
}

/**
 * Builds account metas for `channel_epoch_create` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createChannelEpochCreateAccountMetas = (
  accounts: ChannelEpochCreateAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.channelGroup, isWritable: false, isSigner: false },
    { pubkey: accounts.channel, isWritable: true, isSigner: false },
    { pubkey: accounts.currentEpoch, isWritable: false, isSigner: false },
    { pubkey: accounts.newChannelEpoch, isWritable: true, isSigner: false },
    { pubkey: accounts.rewardTokenMint, isWritable: false, isSigner: false },
    {
      pubkey: accounts.clearingCreditsMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.rewardPotEscrowAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.clearingCreditsEscrowAccount,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createChannelEpochCreateInstruction = (
  programId: PublicKey,
  accounts: ChannelEpochCreateAccounts,
): TransactionInstruction => {
  const keys = createChannelEpochCreateAccountMetas(accounts)
  const data = encodeData(
    CHANNEL_EPOCH_CREATE_INSTRUCTION_SCHEMA,
    CHANNEL_EPOCH_CREATE_INSTRUCTION_DISCRIMINATOR,
    {},
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface ChannelEpochFinalizeAccounts {
  channel: PublicKey
  completedEpoch: PublicKey
  completedChannelEpoch: PublicKey
  clearingCreditsMint: PublicKey
  clearingCreditsEscrowTokenAccount: PublicKey
}

/**
 * Builds account metas for `channel_epoch_finalize` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createChannelEpochFinalizeAccountMetas = (
  accounts: ChannelEpochFinalizeAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.channel, isWritable: false, isSigner: false },
    { pubkey: accounts.completedEpoch, isWritable: false, isSigner: false },
    {
      pubkey: accounts.completedChannelEpoch,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: accounts.clearingCreditsMint, isWritable: true, isSigner: false },
    {
      pubkey: accounts.clearingCreditsEscrowTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createChannelEpochFinalizeInstruction = (
  programId: PublicKey,
  accounts: ChannelEpochFinalizeAccounts,
): TransactionInstruction => {
  const keys = createChannelEpochFinalizeAccountMetas(accounts)
  const data = encodeData(
    CHANNEL_EPOCH_FINALIZE_INSTRUCTION_SCHEMA,
    CHANNEL_EPOCH_FINALIZE_INSTRUCTION_DISCRIMINATOR,
    {},
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface ChannelSetDefaultEpochTermsAccounts {
  authority: PublicKey
  organization: PublicKey
  channelGroup: PublicKey
  channel: PublicKey
}

/**
 * Builds account metas for `channel_set_default_epoch_terms` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createChannelSetDefaultEpochTermsAccountMetas = (
  accounts: ChannelSetDefaultEpochTermsAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.organization, isWritable: false, isSigner: false },
    { pubkey: accounts.channelGroup, isWritable: false, isSigner: false },
    { pubkey: accounts.channel, isWritable: true, isSigner: false },
  ]
}

export const createChannelSetDefaultEpochTermsInstruction = (
  programId: PublicKey,
  accounts: ChannelSetDefaultEpochTermsAccounts,
  args: ChannelSetDefaultEpochTermsInstructionArgs,
): TransactionInstruction => {
  const keys = createChannelSetDefaultEpochTermsAccountMetas(accounts)
  const data = encodeData(
    CHANNEL_SET_DEFAULT_EPOCH_TERMS_INSTRUCTION_SCHEMA,
    CHANNEL_SET_DEFAULT_EPOCH_TERMS_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface ChannelSetDisabledAccounts {
  authority: PublicKey
  organization: PublicKey
  channelGroup: PublicKey
  channel: PublicKey
}

/**
 * Builds account metas for `channel_set_disabled` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createChannelSetDisabledAccountMetas = (
  accounts: ChannelSetDisabledAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.organization, isWritable: false, isSigner: false },
    { pubkey: accounts.channelGroup, isWritable: false, isSigner: false },
    { pubkey: accounts.channel, isWritable: true, isSigner: false },
  ]
}

export const createChannelSetDisabledInstruction = (
  programId: PublicKey,
  accounts: ChannelSetDisabledAccounts,
  args: ChannelSetDisabledInstructionArgs,
): TransactionInstruction => {
  const keys = createChannelSetDisabledAccountMetas(accounts)
  const data = encodeData(
    CHANNEL_SET_DISABLED_INSTRUCTION_SCHEMA,
    CHANNEL_SET_DISABLED_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface CreateChannelGroupAccounts {
  payer: PublicKey
  authority: PublicKey
  organization: PublicKey
  channelGroup: PublicKey
  initialEpoch: PublicKey
  clearingCreditsMint: PublicKey
  clearingCreditsEscrow: PublicKey
}

/**
 * Builds account metas for `create_channel_group` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createCreateChannelGroupAccountMetas = (
  accounts: CreateChannelGroupAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.organization, isWritable: true, isSigner: false },
    { pubkey: accounts.channelGroup, isWritable: true, isSigner: false },
    { pubkey: accounts.initialEpoch, isWritable: true, isSigner: false },
    {
      pubkey: accounts.clearingCreditsMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.clearingCreditsEscrow,
      isWritable: true,
      isSigner: false,
    },
    { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false },
    { pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createCreateChannelGroupInstruction = (
  programId: PublicKey,
  accounts: CreateChannelGroupAccounts,
  args: CreateChannelGroupInstructionArgs,
): TransactionInstruction => {
  const keys = createCreateChannelGroupAccountMetas(accounts)
  const data = encodeData(
    CREATE_CHANNEL_GROUP_INSTRUCTION_SCHEMA,
    CREATE_CHANNEL_GROUP_INSTRUCTION_DISCRIMINATOR,
    args,
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface CreateOrganizationAccounts {
  payer: PublicKey
  authority: PublicKey
  organization: PublicKey
}

/**
 * Builds account metas for `create_organization` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createCreateOrganizationAccountMetas = (
  accounts: CreateOrganizationAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.authority, isWritable: false, isSigner: true },
    { pubkey: accounts.organization, isWritable: true, isSigner: false },
    { pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createCreateOrganizationInstruction = (
  programId: PublicKey,
  accounts: CreateOrganizationAccounts,
): TransactionInstruction => {
  const keys = createCreateOrganizationAccountMetas(accounts)
  const data = encodeData(
    CREATE_ORGANIZATION_INSTRUCTION_SCHEMA,
    CREATE_ORGANIZATION_INSTRUCTION_DISCRIMINATOR,
    {},
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

export interface EpochCreateAccounts {
  payer: PublicKey
  channelGroup: PublicKey
  currentEpoch: PublicKey
  newEpoch: PublicKey
}

/**
 * Builds account metas for `epoch_create` in the exact order the program expects.
 * No instruction data is encoded, so the result can be forwarded as remaining accounts.
 */
export const createEpochCreateAccountMetas = (
  accounts: EpochCreateAccounts,
): AccountMeta[] => {
  return [
    { pubkey: accounts.payer, isWritable: true, isSigner: true },
    { pubkey: accounts.channelGroup, isWritable: true, isSigner: false },
    { pubkey: accounts.currentEpoch, isWritable: false, isSigner: false },
    { pubkey: accounts.newEpoch, isWritable: true, isSigner: false },
    { pubkey: SYSTEM_PROGRAM_ID, isWritable: false, isSigner: false },
  ]
}

export const createEpochCreateInstruction = (
  programId: PublicKey,
  accounts: EpochCreateAccounts,
): TransactionInstruction => {
  const keys = createEpochCreateAccountMetas(accounts)
  const data = encodeData(
    EPOCH_CREATE_INSTRUCTION_SCHEMA,
    EPOCH_CREATE_INSTRUCTION_DISCRIMINATOR,
    {},
  )
  return new TransactionInstruction({
    programId,
    keys,
    data: Buffer.from(data),
  })
}

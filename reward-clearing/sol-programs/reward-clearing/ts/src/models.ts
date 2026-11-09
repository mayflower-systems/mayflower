// Generated from an Anchor IDL. Do not edit.

import type { Schema } from "borsh"
import type { PublicKeyBytes } from "./codecs.js"

// constants ---------------------------------------------------------

export const BIDDER_POSITION_SEED: ReadonlyArray<number> = [
  98, 105, 100, 100, 101, 114, 95, 112, 111, 115, 105, 116, 105, 111, 110,
]
export const CHANNEL_EPOCH_CLEARING_CREDITS_ESCROW_SEED: ReadonlyArray<number> =
  [
    99, 104, 97, 110, 110, 101, 108, 95, 101, 112, 111, 99, 104, 95, 99, 114,
    101, 100, 105, 116, 115, 95, 101, 115, 99, 114, 111, 119,
  ]
export const CHANNEL_EPOCH_REWARD_POT_ESCROW_SEED: ReadonlyArray<number> = [
  99, 104, 97, 110, 110, 101, 108, 95, 101, 112, 111, 99, 104, 95, 114, 101,
  119, 97, 114, 100, 95, 112, 111, 116, 95, 101, 115, 99, 114, 111, 119,
]
export const CHANNEL_EPOCH_SEED: ReadonlyArray<number> = [
  99, 104, 97, 110, 110, 101, 108, 95, 101, 112, 111, 99, 104,
]
export const CHANNEL_GROUP_SEED: ReadonlyArray<number> = [
  99, 104, 97, 110, 110, 101, 108, 95, 103, 114, 111, 117, 112,
]
export const CLEARING_TOKEN_ESCROW_SEED: ReadonlyArray<number> = [
  99, 108, 101, 97, 114, 105, 110, 103, 95, 116, 111, 107, 101, 110, 95, 101,
  115, 99, 114, 111, 119,
]
export const EPOCH_SEED: ReadonlyArray<number> = [101, 112, 111, 99, 104]
export const MAX_LOT_COUNT: number = 8
export const ORGANIZATION_SEED: ReadonlyArray<number> = [
  111, 114, 103, 97, 110, 105, 122, 97, 116, 105, 111, 110,
]
export const REWARD_CHANNEL_SEED: ReadonlyArray<number> = [
  114, 101, 119, 97, 114, 100, 95, 99, 104, 97, 110, 110, 101, 108,
]
export const REWARD_STAGING_SEED: ReadonlyArray<number> = [
  114, 101, 119, 97, 114, 100, 95, 115, 116, 97, 103, 105, 110, 103, 95, 116,
  111, 107, 101, 110, 95, 101, 115, 99, 114, 111, 119,
]

// types -------------------------------------------------------------

export interface BidderLot {
  credits: bigint
  sharesQ64f64: bigint
}

export interface BidderPosition {
  authority: PublicKeyBytes
  channel: PublicKeyBytes
  epochCounter: number
  claimed: boolean
  selfBump: ReadonlyArray<number>
  /**
   * Oldest to newest. Allocated space is retained when withdrawals remove lots.
   */
  lots: ReadonlyArray<BidderLot>
}

/**
 * A mint-specific reward channel synchronized by its channel group's UTC schedule.
 */
export interface Channel {
  channelGroup: PublicKeyBytes
  rewardTokenMint: PublicKeyBytes
  /**
   * Channel-owned reward staging vault, initialized when the channel is created.
   */
  rewardStagingTokenAccount: PublicKeyBytes
  defaultEpochTerms: EpochTerms
  /**
   * The most recent epoch opened by the state machine, starting at channel creation.
   */
  lastCreatedEpochCounter: number
  /**
   * Disabling a channel prevents new epochs; existing epochs remain usable.
   */
  disabled: boolean
  /**
   * Creation-time bump retained for channel-owned token transfers.
   */
  selfBump: ReadonlyArray<number>
}

export interface ChannelCreateArgs {
  defaultEpochTerms: EpochTerms
}

export interface ChannelEpoch {
  /**
   * Link to channel
   */
  channel: PublicKeyBytes
  /**
   * Epoch index within the channel's group.
   */
  epochCounter: number
  /**
   * Token account that holds the reward pot for this epoch for this channel
   */
  rewardPotEscrowTokenAccount: PublicKeyBytes
  /**
   * Token account that holds the pledged clearing credits for this epoch for this channel
   */
  clearingCreditsEscrowTokenAccount: PublicKeyBytes
  /**
   * Recorded balance of the clearing credits for this epoch for this channel
   */
  clearingCreditsBalance: bigint
  /**
   * Recorded balance of the reward pot for this epoch for this channel
   */
  rewardPotBalance: bigint
  /**
   * Q64.64 fixed point number of weighted shares in this epoch for this channel
   */
  weightedSharesBalanceQ64f64: bigint
  /**
   * Terms for this epoch
   */
  terms: StoredEpochTerms
  /**
   * Original pot and total shares, frozen at finalization for every claim.
   */
  finalTotals: StoredFinalizedTotals | null
  selfBump: ReadonlyArray<number>
}

/**
 * A numbered channel group with a fixed schedule and a mutable epoch index.
 */
export interface ChannelGroup {
  organization: PublicKeyBytes
  channelGroupIndex: number
  clearingCreditsMint: PublicKeyBytes
  masterStartTimeUnixSeconds: bigint
  epochDurationSeconds: number
  /**
   * Monotonic epoch index for this channel group.
   */
  currentEpochIndex: number
  /**
   * Creation-time bump used only to sign the permissionless burn.
   */
  selfBump: ReadonlyArray<number>
}

export interface ChannelSetDefaultEpochTermsArgs {
  defaultEpochTerms: EpochTerms
}

export interface ChannelSetDisabledArgs {
  disabled: boolean
}

export interface CreateChannelGroupArgs {
  masterStartTimeUnixSeconds: bigint
  epochDurationSeconds: number
}

export interface Epoch {
  channelGroup: PublicKeyBytes
  epochCounter: number
  startTimeUnixSeconds: bigint
  durationSeconds: number
}

export interface EpochTerms {
  /**
   * Ramp bonus slope in micro-basis points
   */
  rampBonusSlopeMbps: number
  /**
   * Exit penalty exponent
   */
  exitPenaltyExponent: number
}

export interface Organization {
  authority: PublicKeyBytes
  nextChannelGroupIndex: number
}

/**
 * An epoch stores the library's exact numeric snapshot, without converting it
 * back to micro-basis points and introducing another rounding step.
 */
export interface StoredEpochTerms {
  rampBonusSlopeQ64f64: bigint
  exitPenaltyExponent: number
}

export interface StoredFinalizedTotals {
  rewardPot: bigint
  weightedSharesQ64f64: bigint
}

// Borsh schemas -----------------------------------------------------

export const BIDDER_LOT_SCHEMA: Schema = {
  struct: { credits: "u64", sharesQ64f64: "u128" },
}
export const BIDDER_POSITION_SCHEMA: Schema = {
  struct: {
    authority: { array: { type: "u8", len: 32 } },
    channel: { array: { type: "u8", len: 32 } },
    epochCounter: "u32",
    claimed: "bool",
    selfBump: { array: { type: "u8", len: 1 } },
    lots: { array: { type: BIDDER_LOT_SCHEMA } },
  },
}
export const EPOCH_TERMS_SCHEMA: Schema = {
  struct: { rampBonusSlopeMbps: "u32", exitPenaltyExponent: "u8" },
}
export const CHANNEL_SCHEMA: Schema = {
  struct: {
    channelGroup: { array: { type: "u8", len: 32 } },
    rewardTokenMint: { array: { type: "u8", len: 32 } },
    rewardStagingTokenAccount: { array: { type: "u8", len: 32 } },
    defaultEpochTerms: EPOCH_TERMS_SCHEMA,
    lastCreatedEpochCounter: "u32",
    disabled: "bool",
    selfBump: { array: { type: "u8", len: 1 } },
  },
}
export const CHANNEL_CREATE_ARGS_SCHEMA: Schema = {
  struct: { defaultEpochTerms: EPOCH_TERMS_SCHEMA },
}
export const STORED_EPOCH_TERMS_SCHEMA: Schema = {
  struct: { rampBonusSlopeQ64f64: "u128", exitPenaltyExponent: "u32" },
}
export const STORED_FINALIZED_TOTALS_SCHEMA: Schema = {
  struct: { rewardPot: "u64", weightedSharesQ64f64: "u128" },
}
export const CHANNEL_EPOCH_SCHEMA: Schema = {
  struct: {
    channel: { array: { type: "u8", len: 32 } },
    epochCounter: "u32",
    rewardPotEscrowTokenAccount: { array: { type: "u8", len: 32 } },
    clearingCreditsEscrowTokenAccount: { array: { type: "u8", len: 32 } },
    clearingCreditsBalance: "u64",
    rewardPotBalance: "u64",
    weightedSharesBalanceQ64f64: "u128",
    terms: STORED_EPOCH_TERMS_SCHEMA,
    finalTotals: { option: STORED_FINALIZED_TOTALS_SCHEMA },
    selfBump: { array: { type: "u8", len: 1 } },
  },
}
export const CHANNEL_GROUP_SCHEMA: Schema = {
  struct: {
    organization: { array: { type: "u8", len: 32 } },
    channelGroupIndex: "u32",
    clearingCreditsMint: { array: { type: "u8", len: 32 } },
    masterStartTimeUnixSeconds: "u64",
    epochDurationSeconds: "u32",
    currentEpochIndex: "u32",
    selfBump: { array: { type: "u8", len: 1 } },
  },
}
export const CHANNEL_SET_DEFAULT_EPOCH_TERMS_ARGS_SCHEMA: Schema = {
  struct: { defaultEpochTerms: EPOCH_TERMS_SCHEMA },
}
export const CHANNEL_SET_DISABLED_ARGS_SCHEMA: Schema = {
  struct: { disabled: "bool" },
}
export const CREATE_CHANNEL_GROUP_ARGS_SCHEMA: Schema = {
  struct: { masterStartTimeUnixSeconds: "u64", epochDurationSeconds: "u32" },
}
export const EPOCH_SCHEMA: Schema = {
  struct: {
    channelGroup: { array: { type: "u8", len: 32 } },
    epochCounter: "u32",
    startTimeUnixSeconds: "u64",
    durationSeconds: "u32",
  },
}
export const ORGANIZATION_SCHEMA: Schema = {
  struct: {
    authority: { array: { type: "u8", len: 32 } },
    nextChannelGroupIndex: "u32",
  },
}

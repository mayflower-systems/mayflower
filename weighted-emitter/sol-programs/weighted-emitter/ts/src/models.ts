// Generated from an Anchor IDL. Do not edit.

import type { Schema } from "borsh"
import type { PublicKeyBytes } from "./codecs.js"

// constants ---------------------------------------------------------

export const MINT_AUTHORITY_SEED: ReadonlyArray<number> = [
  109, 105, 110, 116, 95, 97, 117, 116, 104, 111, 114, 105, 116, 121,
]
export const VOTE_ESCROW_SEED: ReadonlyArray<number> = [
  118, 111, 116, 101, 95, 101, 115, 99, 114, 111, 119,
]
export const VOTE_POSITION_SEED: ReadonlyArray<number> = [
  118, 111, 116, 101, 95, 112, 111, 115, 105, 116, 105, 111, 110,
]

// types -------------------------------------------------------------

export interface CreateEmitterArgs {
  emissionRateQ64f64: bigint
  annualDecayRateQ64f64: bigint
}

/**
 * Canonical schedule and aggregate allocation state for one independent emitter.
 */
export interface Emitter {
  authority: PublicKeyBytes
  voteMint: PublicKeyBytes
  emissionMint: PublicKeyBytes
  mintAuthorityBump: ReadonlyArray<number>
  emissionRateQ64f64: bigint
  annualDecayRateQ64f64: bigint
  lastUpdatedAtUnixSeconds: bigint
  /**
   * Recorded deposits across gauges, independent of unsolicited escrow transfers.
   */
  totalDeposited: bigint
  /**
   * Whole emission allocated to gauges, including amounts not yet minted.
   */
  totalEmitted: bigint
  totalMinted: bigint
  emissionIndexQ192f64: ReadonlyArray<number>
}

export interface Gauge {
  emitter: PublicKeyBytes
  totalDeposited: bigint
  /**
   * Accrued whole emission, including allocations checkpointed by vote changes.
   */
  totalEmitted: bigint
  /**
   * The authority may mint the difference between total_emitted and total_minted.
   */
  totalMinted: bigint
  lastUpdatedAtUnixSeconds: bigint
  emissionIndexSeenQ192f64: ReadonlyArray<number>
}

export interface SetEmitterDecayRateArgs {
  annualDecayRateQ64f64: bigint
}

export interface VotePosition {
  authority: PublicKeyBytes
  gauge: PublicKeyBytes
  voteEscrow: PublicKeyBytes
  /**
   * Recorded principal; external token donations do not add voting weight.
   */
  deposited: bigint
  selfBump: ReadonlyArray<number>
}

// Borsh schemas -----------------------------------------------------

export const CREATE_EMITTER_ARGS_SCHEMA: Schema = {
  struct: { emissionRateQ64f64: "u128", annualDecayRateQ64f64: "u128" },
}
export const EMITTER_SCHEMA: Schema = {
  struct: {
    authority: { array: { type: "u8", len: 32 } },
    voteMint: { array: { type: "u8", len: 32 } },
    emissionMint: { array: { type: "u8", len: 32 } },
    mintAuthorityBump: { array: { type: "u8", len: 1 } },
    emissionRateQ64f64: "u128",
    annualDecayRateQ64f64: "u128",
    lastUpdatedAtUnixSeconds: "u64",
    totalDeposited: "u64",
    totalEmitted: "u64",
    totalMinted: "u64",
    emissionIndexQ192f64: { array: { type: "u8", len: 32 } },
  },
}
export const GAUGE_SCHEMA: Schema = {
  struct: {
    emitter: { array: { type: "u8", len: 32 } },
    totalDeposited: "u64",
    totalEmitted: "u64",
    totalMinted: "u64",
    lastUpdatedAtUnixSeconds: "u64",
    emissionIndexSeenQ192f64: { array: { type: "u8", len: 32 } },
  },
}
export const SET_EMITTER_DECAY_RATE_ARGS_SCHEMA: Schema = {
  struct: { annualDecayRateQ64f64: "u128" },
}
export const VOTE_POSITION_SCHEMA: Schema = {
  struct: {
    authority: { array: { type: "u8", len: 32 } },
    gauge: { array: { type: "u8", len: 32 } },
    voteEscrow: { array: { type: "u8", len: 32 } },
    deposited: "u64",
    selfBump: { array: { type: "u8", len: 1 } },
  },
}

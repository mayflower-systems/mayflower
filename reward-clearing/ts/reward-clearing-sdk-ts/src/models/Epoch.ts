import type { Epoch as EpochAccount } from "@mayflower-sys/reward-clearing-sol-client"
import * as Address from "./Address.ts"
import type { UnixTimestamp } from "./UnixTimestamp.ts"

/**
 * One interval in the schedule of a channel group. The epoch is live from its
 * start until `durationSeconds` later. The end is not part of the epoch.
 */
export interface Epoch {
  readonly channelGroup: Address.Address
  /** The index of this epoch in the schedule of its channel group. */
  readonly epochCounter: number
  readonly startTimeUnixSeconds: UnixTimestamp
  readonly durationSeconds: number
}

export const fromAccount = (x: EpochAccount): Epoch => ({
  channelGroup: Address.fromBytes(x.channelGroup),
  epochCounter: x.epochCounter,
  startTimeUnixSeconds: { seconds: x.startTimeUnixSeconds },
  durationSeconds: x.durationSeconds,
})

export const toAccount = (x: Epoch): EpochAccount => ({
  channelGroup: Address.toBytes(x.channelGroup),
  epochCounter: x.epochCounter,
  startTimeUnixSeconds: x.startTimeUnixSeconds.seconds,
  durationSeconds: x.durationSeconds,
})

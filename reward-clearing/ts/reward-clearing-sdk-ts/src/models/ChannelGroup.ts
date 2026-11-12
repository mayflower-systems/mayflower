import type { ChannelGroup as ChannelGroupAccount } from "@mayflower-sys/reward-clearing-sol-client"
import * as Address from "./Address.ts"
import type { UnixTimestamp } from "./UnixTimestamp.ts"

/**
 * The channels of an organization that share one schedule of epochs and one
 * mint of clearing credits.
 *
 * Epoch `n` starts at `masterStartTimeUnixSeconds + n * epochDurationSeconds`.
 */
export interface ChannelGroup {
  readonly organization: Address.Address
  /** The index of this group in its organization. */
  readonly channelGroupIndex: number
  readonly clearingCreditsMint: Address.Address
  /** The start of epoch 0. */
  readonly masterStartTimeUnixSeconds: UnixTimestamp
  readonly epochDurationSeconds: number
  /**
   * The counter of the newest epoch. It moves when someone creates the next
   * epoch, so the epoch it names can already be over.
   */
  readonly currentEpochIndex: number
  /** The bump seed of this account's address. */
  readonly selfBump: number
}

export const fromAccount = (x: ChannelGroupAccount): ChannelGroup => ({
  organization: Address.fromBytes(x.organization),
  channelGroupIndex: x.channelGroupIndex,
  clearingCreditsMint: Address.fromBytes(x.clearingCreditsMint),
  masterStartTimeUnixSeconds: { seconds: x.masterStartTimeUnixSeconds },
  epochDurationSeconds: x.epochDurationSeconds,
  currentEpochIndex: x.currentEpochIndex,
  selfBump: x.selfBump[0]!,
})

export const toAccount = (x: ChannelGroup): ChannelGroupAccount => ({
  organization: Address.toBytes(x.organization),
  channelGroupIndex: x.channelGroupIndex,
  clearingCreditsMint: Address.toBytes(x.clearingCreditsMint),
  masterStartTimeUnixSeconds: x.masterStartTimeUnixSeconds.seconds,
  epochDurationSeconds: x.epochDurationSeconds,
  currentEpochIndex: x.currentEpochIndex,
  selfBump: [x.selfBump],
})

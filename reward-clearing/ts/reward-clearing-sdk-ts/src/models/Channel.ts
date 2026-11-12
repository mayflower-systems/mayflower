import {
  ok,
  type FixedNumberError,
  type Result,
} from "@mayflower-sys/fixed-number-ts"
import type { Channel as ChannelAccount } from "@mayflower-sys/reward-clearing-sol-client"
import * as Address from "./Address.ts"
import * as EpochTerms from "./EpochTerms.ts"

/**
 * A stream of rewards in one token. A channel belongs to a channel group and
 * follows that group's epoch schedule.
 */
export interface Channel {
  readonly channelGroup: Address.Address
  readonly rewardTokenMint: Address.Address
  /**
   * The channel's staging vault for the reward token. The program creates it
   * with the channel.
   */
  readonly rewardStagingTokenAccount: Address.Address
  /** The terms that each new channel epoch copies. */
  readonly defaultEpochTerms: EpochTerms.EpochTerms
  /**
   * The counter of the newest channel epoch. A channel may skip epochs, so
   * this can be behind the group's current epoch.
   */
  readonly lastCreatedEpochCounter: number
  /**
   * A disabled channel opens no new channel epochs. Its existing ones still
   * work.
   */
  readonly disabled: boolean
  /** The bump seed of this account's address. */
  readonly selfBump: number
}

export const fromAccount = (x: ChannelAccount): Channel => ({
  channelGroup: Address.fromBytes(x.channelGroup),
  rewardTokenMint: Address.fromBytes(x.rewardTokenMint),
  rewardStagingTokenAccount: Address.fromBytes(x.rewardStagingTokenAccount),
  defaultEpochTerms: EpochTerms.fromAccount(x.defaultEpochTerms),
  lastCreatedEpochCounter: x.lastCreatedEpochCounter,
  disabled: x.disabled,
  selfBump: x.selfBump[0]!,
})

/**
 * Fails when the slope of the default terms is not a whole count of
 * micro-basis points.
 */
export const toAccount = (
  x: Channel,
): Result<ChannelAccount, FixedNumberError> => {
  const defaultEpochTerms = EpochTerms.toAccount(x.defaultEpochTerms)
  if (!defaultEpochTerms.ok) return defaultEpochTerms
  return ok({
    channelGroup: Address.toBytes(x.channelGroup),
    rewardTokenMint: Address.toBytes(x.rewardTokenMint),
    rewardStagingTokenAccount: Address.toBytes(x.rewardStagingTokenAccount),
    defaultEpochTerms: defaultEpochTerms.value,
    lastCreatedEpochCounter: x.lastCreatedEpochCounter,
    disabled: x.disabled,
    selfBump: [x.selfBump],
  })
}

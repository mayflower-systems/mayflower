import type { Organization as OrganizationAccount } from "@mayflower-sys/reward-clearing-sol-client"
import * as Address from "./Address.ts"

/** The owner of channel groups. */
export interface Organization {
  /**
   * The signer that creates channel groups and channels, and changes the
   * settings of a channel.
   */
  readonly authority: Address.Address
  /** The index that the next channel group takes. */
  readonly nextChannelGroupIndex: number
}

export const fromAccount = (x: OrganizationAccount): Organization => ({
  authority: Address.fromBytes(x.authority),
  nextChannelGroupIndex: x.nextChannelGroupIndex,
})

export const toAccount = (x: Organization): OrganizationAccount => ({
  authority: Address.toBytes(x.authority),
  nextChannelGroupIndex: x.nextChannelGroupIndex,
})

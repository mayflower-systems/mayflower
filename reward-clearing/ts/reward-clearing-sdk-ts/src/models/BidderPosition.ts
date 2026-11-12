import {
  ok,
  type FixedNumberError,
  type Result,
} from "@mayflower-sys/fixed-number-ts"
import type {
  BidderLot as BidderLotAccount,
  BidderPosition as BidderPositionAccount,
} from "@mayflower-sys/reward-clearing-sol-client"
import * as Address from "./Address.ts"
import * as BidderLot from "./BidderLot.ts"

/** The credits that one bidder holds in one channel epoch. */
export interface BidderPosition {
  /** The bidder. Only this signer deposits, withdraws and claims. */
  readonly authority: Address.Address
  readonly channel: Address.Address
  /** With `channel`, names the channel epoch of this position. */
  readonly epochCounter: number
  /** Set by a claim. The claim instruction also closes the account. */
  readonly claimed: boolean
  /** The bump seed of this account's address. */
  readonly selfBump: number
  /**
   * One lot for each deposit, oldest first. A withdrawal takes from the
   * newest lot first.
   */
  readonly lots: ReadonlyArray<BidderLot.BidderLot>
}

export const fromAccount = (x: BidderPositionAccount): BidderPosition => ({
  authority: Address.fromBytes(x.authority),
  channel: Address.fromBytes(x.channel),
  epochCounter: x.epochCounter,
  claimed: x.claimed,
  selfBump: x.selfBump[0]!,
  lots: x.lots.map(BidderLot.fromAccount),
})

/** Fails when the shares of a lot are not a Q64.64 value. */
export const toAccount = (
  x: BidderPosition,
): Result<BidderPositionAccount, FixedNumberError> => {
  const lots: Array<BidderLotAccount> = []
  for (const lot of x.lots) {
    const account = BidderLot.toAccount(lot)
    if (!account.ok) return account
    lots.push(account.value)
  }
  return ok({
    authority: Address.toBytes(x.authority),
    channel: Address.toBytes(x.channel),
    epochCounter: x.epochCounter,
    claimed: x.claimed,
    selfBump: [x.selfBump],
    lots,
  })
}

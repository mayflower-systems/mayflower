import {
  ok,
  type FixedNumberError,
  type Result,
} from "@mayflower-sys/fixed-number-ts"
import type { BidderLot as BidderLotAccount } from "@mayflower-sys/reward-clearing-sol-client"
import * as BigDecimal from "./BigDecimal.ts"
import type { TokenBalance } from "./TokenBalance.ts"

/**
 * What is left of one deposit. A deposit of credits makes one lot. Its shares
 * are the credits times the entry weight at the time of the deposit.
 */
export interface BidderLot {
  /** The clearing credits that remain in this lot. */
  readonly credits: TokenBalance
  /** The weighted shares that remain in this lot. */
  readonly shares: BigDecimal.BigDecimal
}

export const fromAccount = (x: BidderLotAccount): BidderLot => ({
  credits: { amount: x.credits },
  shares: BigDecimal.fromQ64f64(x.sharesQ64f64),
})

/** Fails when the shares are not a Q64.64 value. */
export const toAccount = (
  x: BidderLot,
): Result<BidderLotAccount, FixedNumberError> => {
  const shares = BigDecimal.toQ64f64(x.shares)
  if (!shares.ok) return shares
  return ok({ credits: x.credits.amount, sharesQ64f64: shares.value })
}

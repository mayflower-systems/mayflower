import {
  ok,
  type FixedNumberError,
  type Result,
} from "@mayflower-sys/fixed-number-ts"
import type { StoredFinalizedTotals } from "@mayflower-sys/reward-clearing-sol-client"
import * as BigDecimal from "./BigDecimal.ts"
import type { TokenBalance } from "./TokenBalance.ts"

/**
 * The totals that finalization freezes for a channel epoch. A claim pays
 * `rewardPot * shares / weightedShares`, rounded down to whole base units.
 * Every claim divides by these same totals, so the order of claims does not
 * change a payout.
 */
export interface FinalizedTotals {
  /** The reward pot at finalization. */
  readonly rewardPot: TokenBalance
  /** The weighted shares of all bidders at finalization. */
  readonly weightedShares: BigDecimal.BigDecimal
}

export const fromAccount = (x: StoredFinalizedTotals): FinalizedTotals => ({
  rewardPot: { amount: x.rewardPot },
  weightedShares: BigDecimal.fromQ64f64(x.weightedSharesQ64f64),
})

/** Fails when the weighted shares are not a Q64.64 value. */
export const toAccount = (
  x: FinalizedTotals,
): Result<StoredFinalizedTotals, FixedNumberError> => {
  const weightedShares = BigDecimal.toQ64f64(x.weightedShares)
  if (!weightedShares.ok) return weightedShares
  return ok({
    rewardPot: x.rewardPot.amount,
    weightedSharesQ64f64: weightedShares.value,
  })
}

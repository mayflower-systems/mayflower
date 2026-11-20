import {
  ok,
  type FixedNumberError,
  type Result,
} from "@mayflower-sys/fixed-number-ts"
import type { Gauge as GaugeAccount } from "@mayflower-sys/weighted-emitter-sol-client"
import * as Address from "./Address.ts"
import * as BigDecimal from "./BigDecimal.ts"
import type { TokenBalance } from "./TokenBalance.ts"
import type { UnixTimestamp } from "./UnixTimestamp.ts"

/** The vote weight and accrued emission of one gauge in an emitter. */
export interface Gauge {
  readonly emitter: Address.Address
  readonly totalDeposited: TokenBalance
  /** Accrued whole emission, including allocations settled by vote changes. */
  readonly totalEmitted: TokenBalance
  /** The authority can mint totalEmitted minus totalMinted. */
  readonly totalMinted: TokenBalance
  readonly lastUpdatedAtUnixSeconds: UnixTimestamp
  readonly emissionIndexSeen: BigDecimal.BigDecimal
}

export const fromAccount = (x: GaugeAccount): Gauge => ({
  emitter: Address.fromBytes(x.emitter),
  totalDeposited: { amount: x.totalDeposited },
  totalEmitted: { amount: x.totalEmitted },
  totalMinted: { amount: x.totalMinted },
  lastUpdatedAtUnixSeconds: { seconds: x.lastUpdatedAtUnixSeconds },
  emissionIndexSeen: BigDecimal.fromQ192f64(x.emissionIndexSeenQ192f64),
})

/** Fails when the index cannot be stored exactly as Q192.64. */
export const toAccount = (x: Gauge): Result<GaugeAccount, FixedNumberError> => {
  const emissionIndexSeen = BigDecimal.toQ192f64(x.emissionIndexSeen)
  if (!emissionIndexSeen.ok) return emissionIndexSeen
  return ok({
    emitter: Address.toBytes(x.emitter),
    totalDeposited: x.totalDeposited.amount,
    totalEmitted: x.totalEmitted.amount,
    totalMinted: x.totalMinted.amount,
    lastUpdatedAtUnixSeconds: x.lastUpdatedAtUnixSeconds.seconds,
    emissionIndexSeenQ192f64: emissionIndexSeen.value,
  })
}

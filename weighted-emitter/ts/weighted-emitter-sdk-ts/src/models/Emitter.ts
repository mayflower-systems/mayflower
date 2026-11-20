import {
  ok,
  type FixedNumberError,
  type Result,
} from "@mayflower-sys/fixed-number-ts"
import type { Emitter as EmitterAccount } from "@mayflower-sys/weighted-emitter-sol-client"
import * as Address from "./Address.ts"
import * as BigDecimal from "./BigDecimal.ts"
import type { TokenBalance } from "./TokenBalance.ts"
import type { UnixTimestamp } from "./UnixTimestamp.ts"

/** One independent emission schedule and its aggregate gauge allocations. */
export interface Emitter {
  readonly authority: Address.Address
  readonly voteMint: Address.Address
  readonly emissionMint: Address.Address
  readonly mintAuthorityBump: number
  /** Emission token base units per second at the last update. */
  readonly emissionRate: BigDecimal.BigDecimal
  readonly annualDecayRate: BigDecimal.BigDecimal
  readonly lastUpdatedAtUnixSeconds: UnixTimestamp
  /** Recorded vote deposits; unsolicited escrow transfers add no weight. */
  readonly totalDeposited: TokenBalance
  /** Whole emission allocated to gauges, including amounts not yet minted. */
  readonly totalEmitted: TokenBalance
  readonly totalMinted: TokenBalance
  /** Cumulative emission per unit of vote weight. */
  readonly emissionIndex: BigDecimal.BigDecimal
}

export const fromAccount = (x: EmitterAccount): Emitter => ({
  authority: Address.fromBytes(x.authority),
  voteMint: Address.fromBytes(x.voteMint),
  emissionMint: Address.fromBytes(x.emissionMint),
  mintAuthorityBump: x.mintAuthorityBump[0]!,
  emissionRate: BigDecimal.fromQ64f64(x.emissionRateQ64f64),
  annualDecayRate: BigDecimal.fromQ64f64(x.annualDecayRateQ64f64),
  lastUpdatedAtUnixSeconds: { seconds: x.lastUpdatedAtUnixSeconds },
  totalDeposited: { amount: x.totalDeposited },
  totalEmitted: { amount: x.totalEmitted },
  totalMinted: { amount: x.totalMinted },
  emissionIndex: BigDecimal.fromQ192f64(x.emissionIndexQ192f64),
})

/** Fails when a rate or index cannot be stored exactly in its fixed format. */
export const toAccount = (
  x: Emitter,
): Result<EmitterAccount, FixedNumberError> => {
  const emissionRate = BigDecimal.toQ64f64(x.emissionRate)
  if (!emissionRate.ok) return emissionRate
  const annualDecayRate = BigDecimal.toQ64f64(x.annualDecayRate)
  if (!annualDecayRate.ok) return annualDecayRate
  const emissionIndex = BigDecimal.toQ192f64(x.emissionIndex)
  if (!emissionIndex.ok) return emissionIndex
  return ok({
    authority: Address.toBytes(x.authority),
    voteMint: Address.toBytes(x.voteMint),
    emissionMint: Address.toBytes(x.emissionMint),
    mintAuthorityBump: [x.mintAuthorityBump],
    emissionRateQ64f64: emissionRate.value,
    annualDecayRateQ64f64: annualDecayRate.value,
    lastUpdatedAtUnixSeconds: x.lastUpdatedAtUnixSeconds.seconds,
    totalDeposited: x.totalDeposited.amount,
    totalEmitted: x.totalEmitted.amount,
    totalMinted: x.totalMinted.amount,
    emissionIndexQ192f64: emissionIndex.value,
  })
}

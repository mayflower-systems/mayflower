import {
  ok,
  type FixedNumberError,
  type Result,
} from "@mayflower-sys/fixed-number-ts"
import type { ChannelEpoch as ChannelEpochAccount } from "@mayflower-sys/reward-clearing-sol-client"
import * as Address from "./Address.ts"
import * as BigDecimal from "./BigDecimal.ts"
import * as EpochTerms from "./EpochTerms.ts"
import * as FinalizedTotals from "./FinalizedTotals.ts"
import type { TokenBalance } from "./TokenBalance.ts"

/**
 * One epoch of one channel. Bidders deposit clearing credits for weighted
 * shares. After the epoch ends, finalization freezes the totals, and each
 * bidder claims a part of the reward pot in proportion to their shares.
 */
export interface ChannelEpoch {
  readonly channel: Address.Address
  /** The index of the epoch in the schedule of the channel's group. */
  readonly epochCounter: number
  /** The token account that holds the reward pot. */
  readonly rewardPotEscrowTokenAccount: Address.Address
  /** The token account that holds the deposited clearing credits. */
  readonly clearingCreditsEscrowTokenAccount: Address.Address
  /**
   * The clearing credits that bidders deposited and did not withdraw.
   * Finalization burns the credits and keeps this record.
   */
  readonly clearingCreditsBalance: TokenBalance
  /**
   * The rewards that the program accepted for this channel epoch, less the
   * claims it paid.
   */
  readonly rewardPotBalance: TokenBalance
  /** The weighted shares of all bidders that have not claimed. */
  readonly weightedSharesBalance: BigDecimal.BigDecimal
  /** The terms that this channel epoch copied when the program created it. */
  readonly terms: EpochTerms.EpochTerms
  /** `null` until finalization. */
  readonly finalTotals: FinalizedTotals.FinalizedTotals | null
  /** The bump seed of this account's address. */
  readonly selfBump: number
}

export const fromAccount = (x: ChannelEpochAccount): ChannelEpoch => ({
  channel: Address.fromBytes(x.channel),
  epochCounter: x.epochCounter,
  rewardPotEscrowTokenAccount: Address.fromBytes(x.rewardPotEscrowTokenAccount),
  clearingCreditsEscrowTokenAccount: Address.fromBytes(
    x.clearingCreditsEscrowTokenAccount,
  ),
  clearingCreditsBalance: { amount: x.clearingCreditsBalance },
  rewardPotBalance: { amount: x.rewardPotBalance },
  weightedSharesBalance: BigDecimal.fromQ64f64(x.weightedSharesBalanceQ64f64),
  terms: EpochTerms.fromStored(x.terms),
  finalTotals:
    x.finalTotals === null ? null : FinalizedTotals.fromAccount(x.finalTotals),
  selfBump: x.selfBump[0]!,
})

/**
 * Fails when the weighted shares, the slope of the terms, or the final
 * weighted shares are not Q64.64 values.
 */
export const toAccount = (
  x: ChannelEpoch,
): Result<ChannelEpochAccount, FixedNumberError> => {
  const weightedSharesBalance = BigDecimal.toQ64f64(x.weightedSharesBalance)
  if (!weightedSharesBalance.ok) return weightedSharesBalance
  const terms = EpochTerms.toStored(x.terms)
  if (!terms.ok) return terms
  const finalTotals =
    x.finalTotals === null ? ok(null) : FinalizedTotals.toAccount(x.finalTotals)
  if (!finalTotals.ok) return finalTotals
  return ok({
    channel: Address.toBytes(x.channel),
    epochCounter: x.epochCounter,
    rewardPotEscrowTokenAccount: Address.toBytes(x.rewardPotEscrowTokenAccount),
    clearingCreditsEscrowTokenAccount: Address.toBytes(
      x.clearingCreditsEscrowTokenAccount,
    ),
    clearingCreditsBalance: x.clearingCreditsBalance.amount,
    rewardPotBalance: x.rewardPotBalance.amount,
    weightedSharesBalanceQ64f64: weightedSharesBalance.value,
    terms: terms.value,
    finalTotals: finalTotals.value,
    selfBump: [x.selfBump],
  })
}

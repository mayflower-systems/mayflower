import {
  ok,
  type FixedNumberError,
  type Result,
} from "@mayflower-sys/fixed-number-ts"
import type {
  EpochTerms as EpochTermsAccount,
  StoredEpochTerms,
} from "@mayflower-sys/reward-clearing-sol-client"
import * as BigDecimal from "./BigDecimal.ts"

/**
 * The entry and exit curves of a channel epoch. Below, `t` is the time since
 * the epoch started and `T` is the epoch's duration.
 *
 * The program stores the slope in two formats. A channel holds its default
 * terms in micro-basis points: the generated `EpochTerms`. A channel epoch
 * holds the Q64.64 value that the program computed from them, rounded down:
 * the generated `StoredEpochTerms`. Both read as the same model.
 */
export interface EpochTerms {
  /**
   * The slope `b` of the entry bonus, from 0 to 2. A deposit weighs
   * `1 + b * (1 - t / T)` shares per credit: `1 + b` at the start of the
   * epoch, falling in a straight line toward 1 at the end.
   */
  readonly rampBonusSlope: BigDecimal.BigDecimal
  /**
   * The exponent `k` of the exit penalty: 1, 2, 3 or 4. A withdrawal forfeits
   * the fraction `(t / T)^k` of its credits.
   */
  readonly exitPenaltyExponent: number
}

/** Reads the default terms of a channel. */
export const fromAccount = (x: EpochTermsAccount): EpochTerms => ({
  rampBonusSlope: BigDecimal.fromMicroBasisPoints(x.rampBonusSlopeMbps),
  exitPenaltyExponent: x.exitPenaltyExponent,
})

/**
 * Writes the default terms of a channel. Fails when the slope is not a whole
 * count of micro-basis points.
 */
export const toAccount = (
  x: EpochTerms,
): Result<EpochTermsAccount, FixedNumberError> => {
  const slope = BigDecimal.toMicroBasisPoints(x.rampBonusSlope)
  if (!slope.ok) return slope
  return ok({
    rampBonusSlopeMbps: slope.value,
    exitPenaltyExponent: x.exitPenaltyExponent,
  })
}

/** Reads the terms of a channel epoch. */
export const fromStored = (x: StoredEpochTerms): EpochTerms => ({
  rampBonusSlope: BigDecimal.fromQ64f64(x.rampBonusSlopeQ64f64),
  exitPenaltyExponent: x.exitPenaltyExponent,
})

/**
 * Writes the terms of a channel epoch. Fails when the slope is not a Q64.64
 * value.
 */
export const toStored = (
  x: EpochTerms,
): Result<StoredEpochTerms, FixedNumberError> => {
  const slope = BigDecimal.toQ64f64(x.rampBonusSlope)
  if (!slope.ok) return slope
  return ok({
    rampBonusSlopeQ64f64: slope.value,
    exitPenaltyExponent: x.exitPenaltyExponent,
  })
}

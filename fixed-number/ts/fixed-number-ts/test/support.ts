import assert from "node:assert/strict"
import { Decimal } from "decimal.js"
import {
  DecimalNumber,
  I64F64,
  I64Scaled,
  U64F64,
  U128F128,
  U192F64,
  U256Scaled,
  type FixedDriver,
  type FixedNumberError,
  type Result,
  type Rounding,
} from "../src/index.ts"

export const pico = I64Scaled(1_000_000_000_000n)

/** The drivers under test. The Rust fixtures hold the same list in this order. */
export const drivers = [
  I64F64,
  U64F64,
  U192F64,
  U128F128,
  pico,
  I64Scaled(8n),
  U256Scaled(1_000_000_000_000n),
  U256Scaled(250n),
]

/**
 * Asserts the result of each rounding for one number, through both
 * `fromDecimalNumber` and `fromDecimalJs`.
 */
export const assertRounds = (
  driver: FixedDriver,
  number: DecimalNumber,
  results: Record<Rounding, Result<bigint, FixedNumberError>>,
): void => {
  const text = DecimalNumber.format(number)
  for (const rounding of ["exact", "floor", "ceil"] as const) {
    const label = `${driver.name} ${rounding} ${text}`
    assert.deepEqual(
      driver.fromDecimalNumber(number, rounding),
      results[rounding],
      label,
    )
    assert.deepEqual(
      driver.fromDecimalJs(new Decimal(text), rounding),
      results[rounding],
      label,
    )
  }
}

import {
  ok,
  U64F64,
  U192F64,
  type FixedNumberError,
  type Result,
} from "@mayflower-sys/fixed-number-ts"

/**
 * An exact decimal number: `value / 10^decimals`.
 *
 * `{ value: 125n, decimals: 2 }` is 1.25. This plain object has the shape
 * accepted by DecimalNumber.format and the fixed-number drivers.
 */
export interface BigDecimal {
  readonly value: bigint
  /** A non-negative integer count of digits after the decimal point. */
  readonly decimals: number
}

/** Reads a u128 Q64.64 rate exactly, with 64 decimal places. */
export const fromQ64f64 = (raw: bigint): BigDecimal =>
  U64F64.toDecimalNumber(raw)

/** Fails for negative, overflowing, or inexact Q64.64 values; never rounds. */
export const toQ64f64 = (x: BigDecimal): Result<bigint, FixedNumberError> =>
  U64F64.fromDecimalNumber(x)

/** Reads the program's 32 little-endian bytes as an exact Q192.64 index. */
export const fromQ192f64 = (bytes: ReadonlyArray<number>): BigDecimal =>
  U192F64.toDecimalNumber(U192F64.fromLeBytes(Uint8Array.from(bytes)))

/** Writes an exact Q192.64 index as the generated model's little-endian bytes. */
export const toQ192f64 = (
  x: BigDecimal,
): Result<ReadonlyArray<number>, FixedNumberError> => {
  const raw = U192F64.fromDecimalNumber(x)
  if (!raw.ok) return raw
  return ok(Array.from(U192F64.toLeBytes(raw.value)))
}

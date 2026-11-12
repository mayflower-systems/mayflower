import {
  makeDriver,
  ok,
  U64F64,
  type FixedNumberError,
  type Result,
} from "@mayflower-sys/fixed-number-ts"

/**
 * An exact decimal number as plain data: `value / 10^decimals`.
 *
 * `{ value: 123n, decimals: 2 }` is the number 1.23. It has the shape of
 * `DecimalNumber` in `@mayflower-sys/fixed-number-ts`, so the functions of
 * that package accept it. `DecimalNumber.format` writes it as text.
 *
 * The program stores a fraction in one of two formats: Q64.64, or
 * micro-basis points. Reading either format is exact. Writing fails for a
 * number that the format cannot hold.
 */
export interface BigDecimal {
  readonly value: bigint
  /** The count of digits after the decimal point. A non-negative integer. */
  readonly decimals: number
}

/**
 * The format of a rate in micro-basis points: a `u32` count, where 1,000,000
 * make one whole unit.
 */
const MicroBasisPoints = makeDriver({
  name: "MicroBasisPoints",
  signed: false,
  byteLength: 4,
  one: 1_000_000n,
})

/**
 * The number that a raw Q64.64 value stands for: `raw / 2^64`. The program
 * stores the raw value as a `u128`. The result has 64 decimals.
 */
export const fromQ64f64 = (raw: bigint): BigDecimal =>
  U64F64.toDecimalNumber(raw)

/**
 * The raw Q64.64 value of a number. Fails when the number is not a multiple
 * of `2^-64`, is negative, or is `2^64` or more.
 */
export const toQ64f64 = (x: BigDecimal): Result<bigint, FixedNumberError> =>
  U64F64.fromDecimalNumber(x)

/**
 * The number that a count of micro-basis points stands for:
 * `raw / 1,000,000`. The result has 6 decimals.
 */
export const fromMicroBasisPoints = (raw: number): BigDecimal =>
  MicroBasisPoints.toDecimalNumber(BigInt(raw))

/**
 * The count of micro-basis points in a number. Fails when the number is not a
 * whole count, is negative, or does not fit a `u32`.
 */
export const toMicroBasisPoints = (
  x: BigDecimal,
): Result<number, FixedNumberError> => {
  const raw = MicroBasisPoints.fromDecimalNumber(x)
  if (!raw.ok) return raw
  return ok(Number(raw.value))
}

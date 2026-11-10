import { makeDriver, type FixedDriver } from "./driver.ts"

const I64_MAX = (1n << 63n) - 1n
const U64_MAX = (1n << 64n) - 1n

/** Throws unless Rust accepts `scale` as the `SCALE` of the driver `type`. */
const checkScale = (type: string, scale: bigint, largest: bigint): void => {
  if (scale < 1n || scale > largest) {
    throw new RangeError(`${type}: the scale ${scale} is outside 1..${largest}`)
  }
}

/**
 * The Rust driver `I64Scaled<SCALE>`: an `i64` count of quanta, 8 bytes,
 * two's complement. The quantum is `1 / scale`.
 *
 * `scale` is the Rust `SCALE`: the number of quanta in one whole unit, not a
 * count of decimal places. It lies in `1..=i64::MAX`. It must also divide a
 * power of ten, because only such a grid has exact decimal values. This
 * function throws a `RangeError` for any other scale.
 */
export const I64Scaled = (scale: bigint): FixedDriver => {
  checkScale("I64Scaled", scale, I64_MAX)
  return makeDriver({
    name: `I64Scaled<${scale}>`,
    signed: true,
    byteLength: 8,
    one: scale,
  })
}

/**
 * The Rust driver `U256Scaled<SCALE>`: an unsigned 256-bit count of quanta,
 * 32 bytes. The quantum is `1 / scale`.
 *
 * `scale` is the Rust `SCALE`: the number of quanta in one whole unit, not a
 * count of decimal places. It lies in `1..=u64::MAX`. It must also divide a
 * power of ten, because only such a grid has exact decimal values. This
 * function throws a `RangeError` for any other scale.
 */
export const U256Scaled = (scale: bigint): FixedDriver => {
  checkScale("U256Scaled", scale, U64_MAX)
  return makeDriver({
    name: `U256Scaled<${scale}>`,
    signed: false,
    byteLength: 32,
    one: scale,
  })
}

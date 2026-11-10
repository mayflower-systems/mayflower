import { makeDriver } from "./driver.ts"

/**
 * Signed Q64.64. Rust stores it as an `i128`: 16 bytes, two's complement.
 * The quantum is `2^-64` and the range is `[-2^63, 2^63)`.
 */
export const I64F64 = makeDriver({
  name: "I64F64",
  signed: true,
  byteLength: 16,
  one: 1n << 64n,
})

/**
 * Unsigned Q64.64. Rust stores it as a `u128`: 16 bytes. The quantum is
 * `2^-64` and the range is `[0, 2^64)`.
 */
export const U64F64 = makeDriver({
  name: "U64F64",
  signed: false,
  byteLength: 16,
  one: 1n << 64n,
})

/**
 * Unsigned Q192.64. Rust stores it in 32 bytes. The quantum is `2^-64` and the
 * range is `[0, 2^192)`.
 */
export const U192F64 = makeDriver({
  name: "U192F64",
  signed: false,
  byteLength: 32,
  one: 1n << 64n,
})

/**
 * Unsigned Q128.128. Rust stores it in 32 bytes. The quantum is `2^-128` and
 * the range is `[0, 2^128)`.
 */
export const U128F128 = makeDriver({
  name: "U128F128",
  signed: false,
  byteLength: 32,
  one: 1n << 128n,
})

/**
 * A whole number of a token's base units. The program stores it as a `u64`.
 *
 * The mint's decimals turn base units into a display amount. This package
 * does not read mints.
 */
export interface TokenBalance {
  readonly amount: bigint
}

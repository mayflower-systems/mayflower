/**
 * A decimal number as plain data: `value / 10^decimals`.
 *
 * `{ value: 123n, decimals: 2 }` is the number 1.23. One number has many
 * forms: `{ value: 1230n, decimals: 3 }` is also 1.23.
 */
export interface DecimalNumber {
  readonly value: bigint
  /** A non-negative integer. */
  readonly decimals: number
}

/**
 * Throws a `RangeError` unless `decimals` is a non-negative integer. The type
 * `number` cannot say so, and a fraction or a negative count has no meaning.
 */
export const checkDecimals = ({ decimals }: DecimalNumber): void => {
  if (!Number.isSafeInteger(decimals) || decimals < 0) {
    throw new RangeError(
      `decimals must be a non-negative integer, got ${decimals}`,
    )
  }
}

/**
 * The number as decimal text with no exponent and no trailing zeros:
 * `{ value: -1230n, decimals: 3 }` becomes `"-1.23"`. Every form of one number
 * gives the same text.
 */
const format = (number: DecimalNumber): string => {
  checkDecimals(number)
  const { value, decimals } = number
  const negative = value < 0n
  // Pad so that at least one digit stands before the decimal point.
  const digits = (negative ? -value : value)
    .toString()
    .padStart(decimals + 1, "0")
  const whole = digits.slice(0, digits.length - decimals)
  const fraction = digits.slice(digits.length - decimals).replace(/0+$/, "")
  const text = fraction === "" ? whole : `${whole}.${fraction}`
  return negative ? `-${text}` : text
}

export const DecimalNumber = { format } as const

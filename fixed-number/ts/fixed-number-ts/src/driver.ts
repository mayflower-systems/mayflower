import { Decimal } from "decimal.js"
import { checkDecimals, DecimalNumber } from "./DecimalNumber.ts"
import { err, ok, type Result } from "./Result.ts"

/**
 * What to do with a decimal number that lies between two values of a driver.
 *
 * - `"exact"` fails with `"Inexact"`.
 * - `"floor"` takes the greatest driver value that is not above the number.
 * - `"ceil"` takes the smallest driver value that is not below the number.
 *
 * Floor and ceil follow numeric order, as the Rust drivers do: the floor of
 * a negative number is further from zero.
 */
export type Rounding = "exact" | "floor" | "ceil"

/**
 * Why a decimal number has no raw value in a driver.
 *
 * - `"NotFinite"`: the `Decimal` is `NaN` or infinite.
 * - `"NegativeToUnsigned"`: the number is negative and the driver is unsigned.
 * - `"Overflow"`: the number, or its rounded value, is outside the driver's
 *   range.
 * - `"Inexact"`: the rounding is `"exact"` and the number is not a driver
 *   value.
 */
export type FixedNumberError =
  | "NotFinite"
  | "NegativeToUnsigned"
  | "Overflow"
  | "Inexact"

/**
 * One fixed-point format of the Rust crate `mayf-fixed-drivers`.
 *
 * A driver value is stored as a raw integer: a count of quanta. The number it
 * stands for is `raw / one`. Borsh decodes a `u128` or `i128` account field
 * straight into that raw `bigint`. A wider driver is stored as little-endian
 * bytes, and `fromLeBytes` turns those into the raw value.
 *
 * Raw-to-decimal conversion is exact. Decimal-to-raw conversion returns a
 * `Result`, because the number may not fit the driver. A function throws a
 * `RangeError` for a defect in the calling program: a raw value outside
 * `min..max`, or a wrong count of bytes.
 */
export interface FixedDriver {
  /** The Rust type, for example `"U64F64"` or `"I64Scaled<1000000000000>"`. */
  readonly name: string
  readonly signed: boolean
  /** Storage size in bytes. */
  readonly byteLength: number
  /** Raw quanta in one whole unit. */
  readonly one: bigint
  /** The smallest raw value. */
  readonly min: bigint
  /** The largest raw value. */
  readonly max: bigint
  /** The digits after the decimal point that write any value exactly. */
  readonly decimals: number

  /** Reads the raw value from storage bytes, as Rust `from_le_bytes` does. */
  readonly fromLeBytes: (bytes: Uint8Array) => bigint
  /** Writes the raw value as storage bytes, as Rust `to_le_bytes` does. */
  readonly toLeBytes: (raw: bigint) => Uint8Array

  /** The exact number, always with `decimals` digits after the point. */
  readonly toDecimalNumber: (raw: bigint) => DecimalNumber
  /** The raw value of a number. `rounding` defaults to `"exact"`. */
  readonly fromDecimalNumber: (
    number: DecimalNumber,
    rounding?: Rounding,
  ) => Result<bigint, FixedNumberError>

  /** The exact number as a decimal.js `Decimal`. */
  readonly toDecimalJs: (raw: bigint) => Decimal
  /** The raw value of a `Decimal`. `rounding` defaults to `"exact"`. */
  readonly fromDecimalJs: (
    decimal: Decimal,
    rounding?: Rounding,
  ) => Result<bigint, FixedNumberError>
}

export interface Layout {
  readonly name: string
  readonly signed: boolean
  readonly byteLength: number
  /** Raw quanta in one whole unit. At least 1. */
  readonly one: bigint
}

/**
 * The fewest decimal digits that write `1 / one` exactly, or `undefined` when
 * no finite decimal does.
 *
 * `1 / one` is a finite decimal only when `one` divides a power of ten, which
 * means `one = 2^twos * 5^fives`. The smallest such power is
 * `10^max(twos, fives)`.
 */
const decimalPlaces = (one: bigint): number | undefined => {
  let rest = one
  let twos = 0
  let fives = 0
  for (; rest % 2n === 0n; rest /= 2n) twos += 1
  for (; rest % 5n === 0n; rest /= 5n) fives += 1
  return rest === 1n ? Math.max(twos, fives) : undefined
}

/**
 * Divides by `10^decimals` and rounds toward negative infinity. `exact` is
 * true when the division leaves no remainder.
 */
const floorDivPow10 = (
  numerator: bigint,
  decimals: number,
): { floor: bigint; exact: boolean } => {
  // A numerator of n digits is smaller in size than 10^n. A divisor that
  // large leaves a quotient strictly between -1 and 1. Answering that case
  // here also keeps a huge `decimals` from building a huge power of ten.
  const digits = (numerator < 0n ? -numerator : numerator).toString().length
  if (decimals >= digits) {
    return { floor: numerator < 0n ? -1n : 0n, exact: numerator === 0n }
  }
  // BigInt division truncates toward zero and the remainder takes the sign of
  // the numerator. So a negative remainder marks a quotient that is one above
  // the floor.
  const divisor = 10n ** BigInt(decimals)
  const quotient = numerator / divisor
  const remainder = numerator % divisor
  return {
    floor: remainder < 0n ? quotient - 1n : quotient,
    exact: remainder === 0n,
  }
}

export const makeDriver = ({
  name,
  signed,
  byteLength,
  one,
}: Layout): FixedDriver => {
  const bits = byteLength * 8
  /** The count of bit patterns that the storage holds. */
  const span = 1n << BigInt(bits)
  const [min, max] = signed ? [-span / 2n, span / 2n - 1n] : [0n, span - 1n]

  const decimals = decimalPlaces(one)
  if (decimals === undefined) {
    throw new RangeError(
      `${name}: a grid of ${one} quanta per whole unit has no finite decimal form`,
    )
  }
  /** `raw / one` equals `raw * decimalFactor / 10^decimals`. */
  const decimalFactor = 10n ** BigInt(decimals) / one
  /**
   * Ten to this power is beyond the range: every value is smaller in size
   * than `span / one`.
   */
  const wholeDigits = (span / one).toString().length

  const inRange = (raw: bigint): boolean => min <= raw && raw <= max

  const checkRaw = (raw: bigint): void => {
    if (!inRange(raw)) {
      throw new RangeError(
        `${name}: the raw value ${raw} is outside ${min}..${max}`,
      )
    }
  }

  const fromLeBytes: FixedDriver["fromLeBytes"] = (bytes) => {
    if (bytes.length !== byteLength) {
      throw new RangeError(
        `${name}: expected ${byteLength} bytes, got ${bytes.length}`,
      )
    }
    const unsigned = bytes.reduceRight(
      (value, byte) => (value << 8n) | BigInt(byte),
      0n,
    )
    return signed ? BigInt.asIntN(bits, unsigned) : unsigned
  }

  const toLeBytes: FixedDriver["toLeBytes"] = (raw) => {
    checkRaw(raw)
    // `>>` and `&` read a negative bigint as two's complement with endless
    // sign bits, so a negative raw value needs no special case.
    return Uint8Array.from({ length: byteLength }, (_, index) =>
      Number((raw >> BigInt(index * 8)) & 0xffn),
    )
  }

  const toDecimalNumber: FixedDriver["toDecimalNumber"] = (raw) => {
    checkRaw(raw)
    return { value: raw * decimalFactor, decimals }
  }

  // decimal.js keeps every digit of a string, whatever its `precision`.
  const toDecimalJs: FixedDriver["toDecimalJs"] = (raw) =>
    new Decimal(DecimalNumber.format(toDecimalNumber(raw)))

  /**
   * The raw value of the number `value / 10^places`. The exact raw value is
   * `value * one / 10^places`, which lies between the integers `floor` and
   * `ceil`.
   */
  const quantize = (
    value: bigint,
    places: number,
    rounding: Rounding,
  ): Result<bigint, FixedNumberError> => {
    if (value < 0n && !signed) return err("NegativeToUnsigned")
    const { floor, exact } = floorDivPow10(value * one, places)
    const ceil = exact ? floor : floor + 1n
    switch (rounding) {
      case "floor":
        return inRange(floor) ? ok(floor) : err("Overflow")
      case "ceil":
        return inRange(ceil) ? ok(ceil) : err("Overflow")
      case "exact":
        // A number outside the range overflows, whether or not it is on the
        // grid.
        if (!inRange(floor) || !inRange(ceil)) return err("Overflow")
        return exact ? ok(floor) : err("Inexact")
    }
  }

  const fromDecimalNumber: FixedDriver["fromDecimalNumber"] = (
    number,
    rounding = "exact",
  ) => {
    checkDecimals(number)
    return quantize(number.value, number.decimals, rounding)
  }

  const fromDecimalJs: FixedDriver["fromDecimalJs"] = (
    decimal,
    rounding = "exact",
  ) => {
    if (!decimal.isFinite()) return err("NotFinite")

    // The exponent form, such as "-1.25e+3", holds every digit and stays
    // short for any exponent.
    const [mantissa = "", exponent = ""] = decimal.toExponential().split("e")
    const [lead = "", fraction = ""] = mantissa.split(".")
    // The number is `coefficient * 10^power`.
    const coefficient = BigInt(lead + fraction)
    const power = Number(exponent) - fraction.length

    if (power < 0) return quantize(coefficient, -power, rounding)
    // decimal.js allows exponents up to 9e15. A power this large puts the
    // number outside the range, so answer without building the number.
    if (power >= wholeDigits) {
      return err(
        coefficient < 0n && !signed ? "NegativeToUnsigned" : "Overflow",
      )
    }
    return quantize(coefficient * 10n ** BigInt(power), 0, rounding)
  }

  return {
    name,
    signed,
    byteLength,
    one,
    min,
    max,
    decimals,
    fromLeBytes,
    toLeBytes,
    toDecimalNumber,
    fromDecimalNumber,
    toDecimalJs,
    fromDecimalJs,
  }
}

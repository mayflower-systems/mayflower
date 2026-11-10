/**
 * Checks that need no Rust fixture: values a reader can verify by hand,
 * rounding between two neighbouring driver values, the ends of each range,
 * extreme exponents, and the defects that throw.
 */
import assert from "node:assert/strict"
import { test } from "node:test"
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
} from "../src/index.ts"
import { err, ok } from "../src/Result.ts"
import { assertRounds, drivers, pico } from "./support.ts"

/**
 * The number `raw + numerator / 10^places` quanta of `driver`. With
 * `0 < numerator < 10^places` it lies strictly between the driver values
 * `raw` and `raw + 1`.
 *
 * One quantum is `factor / 10^decimals`. So the number is a whole count,
 * `raw * 10^places + numerator`, of units `factor / 10^(decimals + places)`.
 * Building it takes no rounding and no division by a power of ten, which is
 * the step under test.
 */
const between = (
  driver: FixedDriver,
  raw: bigint,
  numerator: bigint,
  places: number,
): DecimalNumber => {
  const factor = 10n ** BigInt(driver.decimals) / driver.one
  return {
    value: (raw * 10n ** BigInt(places) + numerator) * factor,
    decimals: driver.decimals + places,
  }
}

/** A fixed sequence of 64-bit values (a linear congruential generator). */
const sequence = (seed: bigint): (() => bigint) => {
  let state = seed
  return () => {
    state = (state * 6364136223846793005n + 1442695040888963407n) % (1n << 64n)
    return state
  }
}

test("known values convert to the decimals a reader can check by hand", () => {
  const cases: [FixedDriver, bigint, string][] = [
    [U64F64, 1n << 64n, "1"],
    [U64F64, 3n << 63n, "1.5"],
    // 2^-64 is 5^64 / 10^64.
    [U64F64, 1n, `0.${"0".repeat(19)}${5n ** 64n}`],
    [I64F64, -(1n << 63n), "-0.5"],
    [I64F64, -(1n << 127n), "-9223372036854775808"],
    [U192F64, 1n << 255n, (1n << 191n).toString()],
    [U128F128, 1n << 127n, "0.5"],
    [U128F128, (1n << 256n) - (1n << 127n), `${(1n << 128n) - 1n}.5`],
    [pico, 1n, "0.000000000001"],
    [pico, -1_500_000_000_000n, "-1.5"],
    [I64Scaled(8n), -1n, "-0.125"],
    [U256Scaled(250n), 1n, "0.004"],
  ]
  for (const [driver, raw, text] of cases) {
    assert.equal(driver.toDecimalJs(raw).toFixed(), text, driver.name)
    assert.equal(
      DecimalNumber.format(driver.toDecimalNumber(raw)),
      text,
      driver.name,
    )
  }
})

test("a number between two driver values rounds to the one named", () => {
  for (const driver of drivers) {
    const next = sequence(BigInt(driver.byteLength))
    const span = driver.max - driver.min
    const raws = [driver.min, driver.max - 1n]
    if (driver.signed) raws.push(-1n, 0n)
    for (let count = 0; count < 40; count += 1) {
      // Four 64-bit draws cover the widest driver.
      const wide =
        (((next() << 64n) | next()) << 128n) | (next() << 64n) | next()
      raws.push(driver.min + (wide % span))
    }

    for (const raw of raws) {
      // Just above `raw`, halfway, and just below `raw + 1`.
      for (const [numerator, places] of [
        [1n, 1],
        [1n, 60],
        [5n, 1],
        [10n ** 60n - 1n, 60],
      ] as const) {
        assertRounds(driver, between(driver, raw, numerator, places), {
          exact: err("Inexact"),
          floor: ok(raw),
          ceil: ok(raw + 1n),
        })
      }
    }
  }
})

test("a number just outside the range fits only when rounded inward", () => {
  for (const driver of drivers) {
    assertRounds(driver, between(driver, driver.max, 1n, 30), {
      exact: err("Overflow"),
      floor: ok(driver.max),
      ceil: err("Overflow"),
    })

    // One step of 10^-30 quanta below the smallest value.
    const below = between(driver, driver.min - 1n, 10n ** 30n - 1n, 30)
    assertRounds(
      driver,
      below,
      driver.signed ?
        { exact: err("Overflow"), floor: err("Overflow"), ceil: ok(driver.min) }
      : {
          exact: err("NegativeToUnsigned"),
          floor: err("NegativeToUnsigned"),
          ceil: err("NegativeToUnsigned"),
        },
    )
  }
})

test("extreme exponents answer without building a huge number", () => {
  // decimal.js allows exponents from -9e15 to 9e15.
  const tiny = new Decimal("1e-9000000000000000")
  const huge = new Decimal("9.99e+9000000000000000")

  for (const driver of drivers) {
    assert.deepEqual(driver.fromDecimalJs(tiny, "floor"), ok(0n))
    assert.deepEqual(driver.fromDecimalJs(tiny, "ceil"), ok(1n))
    assert.deepEqual(driver.fromDecimalJs(tiny, "exact"), err("Inexact"))
    assert.deepEqual(driver.fromDecimalJs(huge, "floor"), err("Overflow"))

    assert.deepEqual(
      driver.fromDecimalJs(tiny.neg(), "floor"),
      driver.signed ? ok(-1n) : err("NegativeToUnsigned"),
    )
    assert.deepEqual(
      driver.fromDecimalJs(huge.neg(), "ceil"),
      driver.signed ? err("Overflow") : err("NegativeToUnsigned"),
    )

    const smallest = { value: 1n, decimals: Number.MAX_SAFE_INTEGER }
    assert.deepEqual(driver.fromDecimalNumber(smallest, "floor"), ok(0n))
    assert.deepEqual(driver.fromDecimalNumber(smallest, "ceil"), ok(1n))
  }
})

test("a power of ten converts up to the last one that fits", () => {
  for (const driver of drivers) {
    // decimal.js holds 10^n as one digit and an exponent, so these numbers
    // take the path for whole numbers with trailing zeros.
    const digits = (driver.max / driver.one).toString().length
    const largest = new Decimal(`1e${digits - 1}`)
    const raw = 10n ** BigInt(digits - 1) * driver.one
    assert.deepEqual(driver.fromDecimalJs(largest), ok(raw), driver.name)
    assert.deepEqual(
      driver.fromDecimalJs(largest.neg()),
      driver.signed ? ok(-raw) : err("NegativeToUnsigned"),
      driver.name,
    )
    assert.deepEqual(
      driver.fromDecimalJs(new Decimal(`1e${digits}`)),
      err("Overflow"),
      driver.name,
    )
  }
})

test("negative zero is zero, and a non-finite decimal has no raw value", () => {
  for (const driver of drivers) {
    assert.deepEqual(driver.fromDecimalJs(new Decimal("-0")), ok(0n))
    for (const value of [NaN, Infinity, -Infinity]) {
      assert.deepEqual(
        driver.fromDecimalJs(new Decimal(value), "floor"),
        err("NotFinite"),
      )
    }
  }
})

test("the rounding defaults to exact", () => {
  assert.deepEqual(U64F64.fromDecimalJs(new Decimal("0.1")), err("Inexact"))
  assert.deepEqual(
    U64F64.fromDecimalNumber({ value: 1n, decimals: 1 }),
    err("Inexact"),
  )
})

test("a raw value outside the range, or a wrong byte count, is a defect", () => {
  for (const driver of drivers) {
    for (const raw of [driver.min - 1n, driver.max + 1n]) {
      assert.throws(() => driver.toDecimalNumber(raw), RangeError)
      assert.throws(() => driver.toDecimalJs(raw), RangeError)
      assert.throws(() => driver.toLeBytes(raw), RangeError)
    }
    for (const length of [driver.byteLength - 1, driver.byteLength + 1]) {
      assert.throws(
        () => driver.fromLeBytes(new Uint8Array(length)),
        RangeError,
      )
    }
  }
})

test("a scale that Rust rejects, or that has no decimal form, is a defect", () => {
  const i64Max = (1n << 63n) - 1n
  const u64Max = (1n << 64n) - 1n
  for (const scale of [0n, -1n, i64Max + 1n]) {
    assert.throws(() => I64Scaled(scale), RangeError)
  }
  for (const scale of [0n, -1n, u64Max + 1n]) {
    assert.throws(() => U256Scaled(scale), RangeError)
  }

  // Thirds have no finite decimal form. Neither does a grid of 2^63 - 1 or
  // 2^64 - 1 quanta, the largest scales Rust accepts.
  for (const scale of [3n, 6n, 7n, i64Max]) {
    assert.throws(() => I64Scaled(scale), RangeError)
  }
  assert.throws(() => U256Scaled(u64Max), RangeError)

  // The grids with the most decimal places: 2^63 quanta for U256Scaled, and
  // 2^62 for I64Scaled.
  assert.equal(U256Scaled(1n << 63n).decimals, 63)
  assert.equal(I64Scaled(1n << 62n).decimals, 62)
})

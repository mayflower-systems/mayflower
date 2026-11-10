/**
 * Checks every driver against values that the Rust crate `mayf-fixed-drivers`
 * serialized. `fixtures/mayf-fixed-drivers.json` is written by the Rust
 * crate `mayf-fixed-drivers-fixture-gen`; `pnpm fixtures` writes it again.
 */
import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import { test } from "node:test"
import {
  DecimalNumber,
  type FixedNumberError,
  type Result,
} from "../src/index.ts"
import { err, ok } from "../src/Result.ts"
import { assertRounds, drivers } from "./support.ts"

/** What a Rust driver returned: a raw value, or the name of a `MathError`. */
type Rounded = { ok: string } | { err: string }

interface LimbFixture {
  signed: boolean
  bytes: number
  one: string
  min: string
  max: string
  /** A stored value, its raw integer, and its exact decimal text. */
  values: { leBytes: string; raw: string; decimal: string }[]
  /** A decimal number and what `checked_from_ratio_round` made of it. */
  ratios: { decimal: string; floor: Rounded; ceil: Rounded }[]
}

const fixtures = JSON.parse(
  readFileSync(
    new URL("./fixtures/mayf-fixed-drivers.json", import.meta.url),
    "utf8",
  ),
) as Record<string, LimbFixture>

/** The `DecimalNumber` that fixture text such as `"-1.25"` names. */
const decimalNumber = (text: string): DecimalNumber => {
  const [whole = "", fraction = ""] = text.split(".")
  return { value: BigInt(whole + fraction), decimals: fraction.length }
}

/**
 * The result this package gives where a Rust driver gave `rounded`.
 *
 * `U256Scaled` takes `checked_from_ratio_round` from the `FixedNumber` trait,
 * whose default reports a negative numerator as `Domain`. The other unsigned
 * drivers report `NegativeToUnsigned`, and this package uses that name for
 * every unsigned driver.
 */
const expected = (rounded: Rounded): Result<bigint, FixedNumberError> => {
  if ("ok" in rounded) return ok(BigInt(rounded.ok))
  return err(
    rounded.err === "Domain" ?
      "NegativeToUnsigned"
    : (rounded.err as FixedNumberError),
  )
}

test("the fixtures hold exactly the drivers under test", () => {
  assert.deepEqual(
    Object.keys(fixtures),
    drivers.map((driver) => driver.name),
  )
})

for (const driver of drivers) {
  const fixture = fixtures[driver.name]
  if (fixture === undefined) continue

  test(`${driver.name} has the layout of the Rust driver`, () => {
    assert.equal(driver.signed, fixture.signed)
    assert.equal(driver.byteLength, fixture.bytes)
    assert.equal(driver.one, BigInt(fixture.one))
    assert.equal(driver.min, BigInt(fixture.min))
    assert.equal(driver.max, BigInt(fixture.max))
  })

  test(`${driver.name} reads each stored value as its exact decimal`, () => {
    for (const { leBytes, raw: rawText, decimal } of fixture.values) {
      const raw = BigInt(rawText)
      assert.equal(driver.fromLeBytes(Buffer.from(leBytes, "hex")), raw)
      assert.equal(Buffer.from(driver.toLeBytes(raw)).toString("hex"), leBytes)

      assert.equal(driver.toDecimalJs(raw).toFixed(), decimal)
      const number = driver.toDecimalNumber(raw)
      assert.equal(number.decimals, driver.decimals)
      assert.equal(DecimalNumber.format(number), decimal)
    }
  })

  test(`${driver.name} writes each exact decimal as its stored value`, () => {
    for (const { raw, decimal } of fixture.values) {
      const stored = ok(BigInt(raw))
      assertRounds(driver, decimalNumber(decimal), {
        exact: stored,
        floor: stored,
        ceil: stored,
      })
    }
  })

  test(`${driver.name} rounds a decimal as the Rust driver does`, () => {
    for (const ratio of fixture.ratios) {
      const floor = expected(ratio.floor)
      const ceil = expected(ratio.ceil)
      // An exact conversion fails where either direction fails. Otherwise it
      // succeeds only when both directions agree.
      const exact =
        !floor.ok ? floor
        : !ceil.ok ? ceil
        : floor.value === ceil.value ? floor
        : err("Inexact")
      assertRounds(driver, decimalNumber(ratio.decimal), { exact, floor, ceil })
    }
  })
}

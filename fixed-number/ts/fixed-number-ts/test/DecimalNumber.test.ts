import assert from "node:assert/strict"
import { test } from "node:test"
import { DecimalNumber, U64F64 } from "../src/index.ts"

test("format writes the shortest decimal text of a number", () => {
  const cases: [bigint, number, string][] = [
    [123n, 2, "1.23"],
    [-123n, 2, "-1.23"],
    [1230n, 3, "1.23"],
    [-5n, 3, "-0.005"],
    [1200n, 2, "12"],
    [7n, 0, "7"],
    [0n, 5, "0"],
  ]
  for (const [value, decimals, text] of cases) {
    assert.equal(DecimalNumber.format({ value, decimals }), text)
  }
})

test("decimals that are not a non-negative integer are a defect", () => {
  for (const decimals of [-1, 0.5, NaN, Infinity]) {
    const number = { value: 1n, decimals }
    assert.throws(() => DecimalNumber.format(number), RangeError)
    assert.throws(() => U64F64.fromDecimalNumber(number), RangeError)
  }
})

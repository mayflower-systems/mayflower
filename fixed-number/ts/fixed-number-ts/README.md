# @mayflower-sys/fixed-number-ts

Converts the fixed-point values that the Solana programs store into decimal numbers, and back.

The Rust crate `mayf-fixed-drivers` defines the stored formats. This package has one driver object for each format. Every driver converts to and from two decimal types:

- a [decimal.js](https://mikemcl.github.io/decimal.js/) `Decimal`, for arithmetic;
- a `DecimalNumber`, which is plain data: `{ value: 123n, decimals: 2 }` is 1.23.

## Drivers

| Driver              | Rust storage     | Range                   | Quantum     | `decimals`   |
| ------------------- | ---------------- | ----------------------- | ----------- | ------------ |
| `I64F64`            | `i128`, 16 bytes | `[-2^63, 2^63)`         | `2^-64`     | 64           |
| `U64F64`            | `u128`, 16 bytes | `[0, 2^64)`             | `2^-64`     | 64           |
| `U192F64`           | 32 bytes         | `[0, 2^192)`            | `2^-64`     | 64           |
| `U128F128`          | 32 bytes         | `[0, 2^128)`            | `2^-128`    | 128          |
| `I64Scaled(scale)`  | `i64`, 8 bytes   | `[-2^63, 2^63) / scale` | `1 / scale` | set by scale |
| `U256Scaled(scale)` | 32 bytes         | `[0, 2^256) / scale`    | `1 / scale` | set by scale |

A stored value is a raw integer: a count of quanta. The number it stands for is `raw / driver.one`. This package passes a raw value as a `bigint`.

`decimals` is the number of digits after the decimal point that writes every value of the driver exactly. A binary quantum such as `2^-64` needs 64 digits, because `2^-64 = 5^64 / 10^64`.

## Reading an account

Borsh decodes a `u128` or `i128` field into a `bigint`. That `bigint` is the raw value.

```ts
import { U64F64 } from "@mayflower-sys/fixed-number-ts"

// `sharesQ64f64` is a Borsh `u128`.
const shares = U64F64.toDecimalJs(position.sharesQ64f64) // Decimal
const portable = U64F64.toDecimalNumber(position.sharesQ64f64) // DecimalNumber
```

A 32-byte driver has no Borsh integer type. Read its raw value from the stored bytes. `fromLeBytes` handles the sign of a signed driver.

```ts
import { U192F64 } from "@mayflower-sys/fixed-number-ts"

// Borsh decodes `[u8; 32]` into an array of numbers.
const raw = U192F64.fromLeBytes(Uint8Array.from(account.cumulative)) // bigint
const cumulative = U192F64.toDecimalJs(raw)
```

Both conversions are exact. No digit is rounded or dropped.

`toDecimalNumber` always returns `driver.decimals` digits after the point. Two values of one driver therefore share a scale, and their `value` fields add and compare directly. `DecimalNumber.format(number)` writes a number as text, such as `"-1.23"`.

## Writing a value

A decimal number is usually not a value of a binary driver: 0.1 has no exact form on the `2^-64` grid. So `fromDecimalJs` and `fromDecimalNumber` take a rounding:

- `"exact"` (the default) fails with `"Inexact"` when the number is not a driver value;
- `"floor"` takes the greatest driver value that is not above the number;
- `"ceil"` takes the smallest driver value that is not below the number.

Floor and ceil follow numeric order, as the Rust drivers do. The floor of a negative number is further from zero.

```ts
import { Decimal } from "decimal.js"
import { U64F64 } from "@mayflower-sys/fixed-number-ts"

const result = U64F64.fromDecimalJs(new Decimal("234.123"), "floor")
if (result.ok) {
  const raw = result.value // 4318807062769101352992n
  const bytes = U64F64.toLeBytes(raw) // 16 bytes, as Rust `to_le_bytes`
} else {
  result.error // "NotFinite" | "NegativeToUnsigned" | "Overflow" | "Inexact"
}
```

| Error                  | Meaning                                                          |
| ---------------------- | ---------------------------------------------------------------- |
| `"NotFinite"`          | The `Decimal` is `NaN` or infinite.                              |
| `"NegativeToUnsigned"` | The number is negative and the driver is unsigned.               |
| `"Overflow"`           | The number, or its rounded value, is outside the driver's range. |
| `"Inexact"`            | The rounding is `"exact"` and the number is not a driver value.  |

## Scaled drivers

`I64Scaled` and `U256Scaled` take the Rust `SCALE`: the number of quanta in one whole unit. It is not a count of decimal places.

```ts
import { I64Scaled } from "@mayflower-sys/fixed-number-ts"

const Pico = I64Scaled(1_000_000_000_000n) // Rust: I64Scaled<1_000_000_000_000>
Pico.toDecimalJs(1_500_000_000_000n).toFixed() // "1.5"
```

The scale must divide a power of ten. A grid of thirds has no exact decimal values, so `I64Scaled(3n)` throws a `RangeError`.

## decimal.js

decimal.js is a peer dependency. The package uses the consumer's copy, so a `Decimal` from this package and a `Decimal` from the consumer share one configuration.

`toDecimalJs` returns every digit of the stored value. Arithmetic on the result is a different matter: decimal.js rounds each result to `Decimal.precision` significant digits, and the default is 20. A driver value can have many more:

| Driver     | Most significant digits in one value |
| ---------- | ------------------------------------ |
| `I64F64`   | 83                                   |
| `U64F64`   | 84                                   |
| `U192F64`  | 122                                  |
| `U128F128` | 167                                  |

Raise the precision before doing arithmetic that must keep them:

```ts
Decimal.set({ precision: 200 })
```

## Failures

A failure that a correct program can meet is a returned value. Every decimal-to-raw conversion returns a `Result` for that reason.

A defect in the calling program throws a `RangeError`. The defects are:

- a raw value outside `driver.min..driver.max`;
- storage bytes of the wrong length;
- a `DecimalNumber` whose `decimals` is not a non-negative integer;
- a scale that Rust rejects, or that does not divide a power of ten.

The range check on a raw value catches a signed field decoded as unsigned, and a field passed to the wrong driver.

## Tests

```sh
pnpm test
```

`test/fixtures/mayf-fixed-drivers.json` holds values that the Rust drivers serialized, with the exact decimal text of each. The Rust crate `mayf-fixed-drivers-fixture-gen` writes the file. It finds the decimal text by long division. This package converts by multiplication, so the two sides share no method.

Write the file again after a change to the drivers:

```sh
pnpm fixtures
```

`cargo test` fails when the checked-in file differs from what the drivers produce.

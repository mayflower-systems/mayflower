# mayf-fixed-drivers-fixture-gen

Writes the oracle fixtures for the TypeScript package
`@mayflower-sys/fixed-number-ts`. That package converts serialized
`mayf-fixed-drivers` values into decimal numbers, and its tests check the
conversion against this file.

```sh
cargo run -p mayf-fixed-drivers-fixture-gen
```

The command writes
`fixed-number/ts/fixed-number-ts/test/fixtures/mayf-fixed-drivers.json`.

## Contents

The file has one member for each driver: `I64F64`, `U64F64`, `U192F64`,
`U128F128`, and two scales each of `I64Scaled` and `U256Scaled`.

| Field                             | Source                                                   |
| --------------------------------- | -------------------------------------------------------- |
| `signed`, `bytes`                 | The driver's `MIN` and its `to_le_bytes` width.          |
| `one`, `min`, `max`               | The raw value of the driver's `ONE`, `MIN` and `MAX`.    |
| `values[].leBytes`                | The driver's `to_le_bytes`, as hex.                      |
| `values[].raw`                    | Those bytes read as one integer.                         |
| `values[].decimal`                | The exact decimal text of `raw / one`.                   |
| `ratios[].decimal`                | A decimal number, `numerator / 10^k`.                    |
| `ratios[].floor`, `ratios[].ceil` | The driver's `checked_from_ratio_round` for that number. |

A rounded result is `{ "ok": raw }` or `{ "err": name }`, where `name` is the
`MathError` variant.

The values cover the constants, the bits on both sides of each 64-bit limb
edge, and random byte patterns from a fixed seed. Every run writes the same
file.

## The decimal oracle

No driver prints an exact decimal, so this crate computes the text itself. It
divides `raw` by `one` with Dashu integers and takes one digit per step. The
TypeScript package multiplies by a power of five instead. The two methods share
no code, so an error in one does not hide in the other.

## Staleness

`cargo test` fails when the checked-in file differs from the output of this
crate. Run the command above after any change to a driver's storage or
rounding.

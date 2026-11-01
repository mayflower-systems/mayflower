# mayf-micro-basis-points

The micro-basis-point wire scale, shared by every crate that persists a rate.

A micro-basis point is one hundredth of a basis point:

| Value      | Micro-basis points |
| ---------- | ------------------ |
| 1 bp       | `100`              |
| 1%         | `10_000`           |
| 100% (1.0) | `1_000_000`        |

## Owns

- `MicroBasisPoints`: a `u32` rate proven to lie between zero and one hundred
  percent at construction.
- The conversion onto a `FixedNumber` grid, which divides by `ONE` with an
  explicit rounding direction.
- `raw_to_fixed`, the same division for wire values whose range is wider than a
  percentage and is owned by another layer.

## Does Not Own

This crate knows nothing about fees, epochs, or accounts. It does not choose a
rounding direction for you, and it does not decide which policy bounds a given
rate — a multiplier such as the reward-clearing ramp bonus slope rides this
scale but is validated where its `[0, 2]` range is defined.

## Wire Types Stay Raw

Account layouts and instruction arguments keep their `u32` fields, so adopting
this crate changes no serialized layout and no IDL. Refine at the boundary:
read the raw field, construct a `MicroBasisPoints`, and work with the refined
value from there.

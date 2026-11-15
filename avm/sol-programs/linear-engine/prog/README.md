# `mayf-avm-engine-linear-sol`

Standalone Anchor program for the neutral characteristic `f(x) = slope * x`.
An AVM host can call it for characteristic values and integrals while managing
market state and policy itself.

## Instructions

- `create_linear_engine` creates a parameter account with a positive slope.
  The payer and new account sign. The slope cannot be updated or the account
  closed by this program.
- `calculate_spans` takes a read-only parameter account and a batch of
  `SpanRequest`s. It returns a Borsh `Vec<SpanResponse>` through Solana return
  data, in request order. Each response encloses `f(lo)`, `f(lo + width)`, and
  the integral over that interval. A zero-width span returns equal endpoint
  bounds and exact zero area. An empty batch returns an empty vector.

All wire numbers are unsigned Q64.64 encoded as little-endian `u128`s:
`1 << 64` means `1.0`. The slope and coordinates use this same grid. Callers
normalize token units before calling; responses remain normalized quantities.
The program uses `LocalCharacteristic<LinearCharacteristic<U64F64>>`, sharing
the existing driver's checked arithmetic and outward rounding.

A response is six `u128`s (96 bytes). Solana allows 1,024 bytes of return data,
so a Borsh vector fits up to 10 responses after its four-byte length prefix.
`MAX_BATCH_SPANS` is derived from that transport limit. A batch larger than
this or an arithmetic overflow fails the whole instruction.

## Calling the engine

The sibling `cpi` crate provides generated instruction builders and Borsh
models without depending on this Anchor program crate. See the
[build instructions](../README.md) to regenerate it.

This crate's `cpi` feature also exposes Anchor's generated
`cpi::calculate_spans` instruction. It returns Anchor's
`Return<Vec<wire::SpanResponse>>`. The public wire types convert to and from
the calculator's `SpanRequest<U64F64>` and `SpanResponse<U64F64>`. Decoding
checks that each lower bound is at most its upper bound.

A `CharacteristicEngine` adapter should check input/output lengths, invoke the
batch once, check the response count, and populate the caller's response slice.
For a fallible transport decoder, read return data immediately after invocation,
verify its producer program ID, and deserialize the whole payload. Anchor's
generated `Return::get()` also checks the producer but panics on invalid data.

Bind the engine program and parameter account addresses in the market's stored
configuration. Anchor checks the parameter account's owner and discriminator.
The host authenticates both addresses against the market configuration and
relies on the bound engine to enclose the true values and integrals. The
calculation uses only a read-only parameter account.

The calculator's signed `I64F64` grid has the same fractional precision.
Adapters can use the fixed drivers' checked `TryFrom` conversions between
`I64F64` and `U64F64`. These conversions reject values outside the target
range. The host handles normalization, token rounding, fees, and storage. The
calculator uses the returned bounds to construct the schedule.

## Verification

Run native tests with `cargo test -p mayf-avm-engine-linear-sol --locked`.
Build the binary, IDL, and CPI crate from the repository root:

```bash
./avm/sol-programs/linear-engine/build.sh
```

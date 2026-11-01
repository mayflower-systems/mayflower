# mayf-fixed-drivers

Fixed-point implementations of the contracts in `mayf-fixed-number`, for
production algorithms and tests. Default drivers are `no_std`, allocation-free,
and contain no unsafe code. The optional `reference` feature provides allocating
host arithmetic through Dashu.

| Format     | Storage  | Range           | Quantum  | Capabilities                                                           |
| ---------- | -------- | --------------- | -------- | ---------------------------------------------------------------------- |
| `I64F64`   | 16 bytes | `[-2^63, 2^63)` | `2^-64`  | `FixedNumber`, `SignedFixed`, `IntegralRounding`, `BoundedExponential` |
| `U64F64`   | 16 bytes | `[0, 2^64)`     | `2^-64`  | `FixedNumber`, `UnsignedFixed`, `IntegralRounding`                     |
| `U192F64`  | 32 bytes | `[0, 2^192)`    | `2^-64`  | `FixedNumber`, `UnsignedFixed`, `IntegralRounding`                     |
| `U128F128` | 32 bytes | `[0, 2^128)`    | `2^-128` | Specialized index arithmetic; no `FixedNumber` implementation          |
| `I64Scaled<S>` | 8 bytes | `[-2^63/S, (2^63-1)/S]` | `1/S` | `FixedNumber`, `SignedFixed`, `IntegralRounding` |
| `U256Scaled<S>` | 32 bytes | `[0, (2^256-1)/S]` | `1/S` | `FixedNumber`, `UnsignedFixed`, `IntegralRounding` |
| `reference::DashuFixed12` | 16 bytes | `[-2^127/10^12, (2^127-1)/10^12]` | `1e-12` | `FixedNumber`, `SignedFixed`, `IntegralRounding`, `BoundedExponential` |

Binary upper endpoints are exclusive; scaled ranges include both endpoints.
`I64F64`'s 64 integer bits include its sign.
Every format is an owned transparent newtype with private storage, not an alias
to a dependency's numeric type. `fixed` supplies signed binary storage and exact
operations; `crypto-bigint` supplies stack-allocated wide integers. Local limb
kernels implement directed rounding and fused arithmetic. The signed driver
exposes raw bits and stable bytes for storage; its wrapped `fixed` value stays
private.

## Choosing a grid at build time

Pure algorithms depend only on `mayf-fixed-number` and accept `N: FixedNumber`
(or additional capabilities when required). Consumers choose the concrete `N`
as a type parameter, so Rust specializes the algorithm at build time. External
implementations remain supported; the trait is open and requires no registry,
runtime dispatch, or dependency on this crate.

`I64Scaled<S>` and `U256Scaled<S>` set the grid using a positive number of
quanta per whole unit, not a count of decimal places. Choose `S = 10` for tenths,
`S = 1_000_000_000_000` for pico-units, or another scale for another uniform
grid. `I64Scaled` requires `S <= i64::MAX`; `U256Scaled` accepts every nonzero
`u64` scale. Invalid scales fail constant evaluation on construction.

```rust
use mayf_fixed_drivers::{I64Scaled, U256Scaled};
use mayf_fixed_number::{FixedNumber, MathResult};

fn half<N: FixedNumber>(value: N) -> MathResult<N> {
    value.checked_div_floor(N::checked_from_i128(2)?)
}

type Tenths = I64Scaled<10>;
type WidePico = U256Scaled<1_000_000_000_000>;
assert_eq!(half(Tenths::ONE)?.to_scaled_i64(), 5);
assert_eq!(half(WidePico::ONE)?.checked_to_scaled_u128()?, 500_000_000_000);
# Ok::<(), mayf_fixed_number::MathError>(())
```

Both scaled drivers use private transparent storage, checked arithmetic, wide
intermediates, explicit rounding, and stable little-endian bytes. Use
`from_scaled_i64`/`to_scaled_i64` or
`from_scaled_u128`/`checked_to_scaled_u128` for raw quanta, and
`from_le_bytes`/`to_le_bytes` for their full storage range. Ordinary consumers
need no limb dependency. `U256Scaled` also exposes raw `U256` access for callers
already using that backend. Its general fused arithmetic uses 512-bit
intermediates and variable-time division; benchmark it for your target before
choosing it over a binary driver.

Zero scales and signed scales that cannot represent `ONE` are rejected:

```compile_fail
use mayf_fixed_drivers::U256Scaled;
const INVALID: U256Scaled<0> = U256Scaled::ZERO;
```

```compile_fail
use mayf_fixed_drivers::I64Scaled;
const INVALID: I64Scaled<{u64::MAX}> = I64Scaled::ONE;
```

## Optional reference arithmetic

Tests needing several grids and independent exact arithmetic can declare a
single dependency:

```toml
[dev-dependencies]
mayf-fixed-drivers = { workspace = true, features = ["reference"] }
```

`mayf_fixed_drivers::reference` exposes `DashuFixed12`, the signed `1e-12`
`FixedNumber` implementation, and Dashu's arbitrary-precision `IBig`/`UBig`
integers. Host applications can enable the same feature as a normal dependency.
The integer types let an oracle compute exact numerators and denominators
independently of fixed-point kernels. They are not `FixedNumber` implementations:
unbounded values cannot satisfy `Copy`. `DashuFixed12` has bounded `i128` storage
and allocating intermediates; wide decimal storage is provided by `U256Scaled`.
The reference feature is disabled by default, so production consumers do not
pull in Dashu unless they request it.

## Generic formulas and value constraints

Choose a representation at the application boundary. A generic algorithm asks
for capabilities; `Pos<N>` and `NonNeg<N>` constrain individual values without
converting their representation:

```rust
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{MathResult, Pos, SignedFixed};

fn difference<N: SignedFixed>(amount: Pos<N>, adjustment: N) -> MathResult<N> {
    amount.get().checked_sub(adjustment)
}

let amount = Pos::new(I64F64::from(2_i64))?;
assert_eq!(difference(amount, I64F64::from(3_i64))?, I64F64::from(-1_i64));
# Ok::<(), mayf_fixed_number::MathError>(())
```

Unsigned formats cannot satisfy a signed capability requirement:

```compile_fail
use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::SignedFixed;
fn requires_signed<N: SignedFixed>(_: N) {}
requires_signed(U64F64::ONE);
```

```compile_fail
use mayf_fixed_drivers::U192F64;
use mayf_fixed_number::SignedFixed;
fn requires_signed<N: SignedFixed>(_: N) {}
requires_signed(U192F64::ONE);
```

Likewise, a signed representation does not promise all values are nonnegative:

```compile_fail
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::UnsignedFixed;
fn requires_unsigned<N: UnsignedFixed>(_: N) {}
requires_unsigned(I64F64::ONE);
```

Negation belongs to `SignedFixed`, not `FixedNumber`. There is no `Signed<N>`
wrapper, and unsigned drivers have no negation operation:

```compile_fail
use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::{FixedNumber, SignedFixed};
let _ = U64F64::ONE.checked_neg();
```

Even a generic algorithm must state the signed capability before negating:

```compile_fail
use mayf_fixed_number::{FixedNumber, MathResult, SignedFixed};
fn negate<N: FixedNumber>(value: N) -> MathResult<N> {
    value.checked_neg()
}
```

Refinements have no unchecked constructor that could bypass their predicate:

```compile_fail
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::Pos;
let _ = Pos::new_unchecked(I64F64::ZERO);
```

## Arithmetic and conversions

Addition and subtraction are exact or return an error. Multiplication, division,
ratio construction, and fused multiply-divide take explicit floor/ceiling
directions through `FixedNumber`. Intermediate products are widened, so a
representable fused quotient survives an overflowing standalone product.
Division by zero is always an error, including for a zero numerator. Signed
floor and ceiling follow mathematical order, not truncation toward zero.

Use `Bounded<N>` when a formula must preserve both outward-rounded endpoints:

```rust
use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::FixedNumber;

let value = U64F64::MAX.bounded() * U64F64::MAX / U64F64::MAX;
assert_eq!(value.bounds()?.into_raw().into_edges(), (U64F64::MAX, U64F64::MAX));
# Ok::<(), mayf_fixed_number::MathError>(())
```

`I64F64` provides certified bounds for `exp(x)-1`, including negative arguments.
Its implementation preserves the legacy range-reduction and directed-series
construction. Positive overflow returns an error; extreme negative inputs are
enclosed between `-1` and `-1 + QUANTUM`.

`U128F128` supports checked addition/subtraction, construction from ratios,
scaling by integer ratios, multiplication by `U64F64`, and directed narrowing.
It deliberately does not offer general wide fixed-by-fixed division:

```compile_fail
use mayf_fixed_drivers::U128F128;
use mayf_fixed_number::FixedNumber;
fn requires_full_arithmetic<N: FixedNumber>(_: N) {}
requires_full_arithmetic(U128F128::ONE);
```

Conversions use these conventions:

- `From` is exact and infallible: whole `i64` into `I64F64`, whole `u64` into
  unsigned formats, and `U64F64` into either wider format.
- `TryFrom` is exact and checked: signed/unsigned Q64.64 conversion and wide
  unsigned narrowing to `U64F64`. Negative-to-unsigned, out-of-range, and
  inexact conversions report errors. `U64F64`'s upper half cannot fit in
  `I64F64`.
- Exact conversions between formats use only `From`/`TryFrom`, without named
  aliases. Whole-unit settlement uses `IntegralRounding::checked_floor_to_u64`.
- `U128F128::checked_to_u64f64_floor/ceil` explicitly permit quantization.
- `from_scaled_*` and `to_scaled_*` access raw grid values. A raw `1` is a
  quantum, not one whole unit. `from_le_bytes` and `to_le_bytes` preserve those
  bits exactly, using two's complement for the signed format.

`U192F64` retains the entire `u64 × u64` integral range with 64 fractional bits.
Whole-unit settlement to `u64` remains range checked. The full driver does use
512-bit intermediates for general wide multiplication/division, while integer
operands and small divisors have shorter paths. Wide division is variable-time
and intended for public accounting quantities, not secrets. No target-specific
performance guarantee is made without measurement.

## Verification and adoption

Independent arbitrary-precision integer and exponential oracles check rounding,
overflow, wide intermediates, and enclosures. Additional tests cover capability
bounds, conversion boundaries, storage, and full-width unsigned denominators for
signed ratio construction.

```sh
cargo test -p mayf-fixed-drivers -p mayf-fixed-number
cargo test -p mayf-fixed-drivers --features reference
cargo check -p mayf-fixed-drivers --no-default-features
```

To opt in, depend on `mayf-fixed-drivers`, replace `BinFixed64` with `I64F64`,
and import the required traits from `mayf-fixed-number`, including `SignedFixed`
when negating. The unsigned format names, grids, and byte encodings are
retained. Use `TryFrom` for signed-to-unsigned conversions. No cross-crate
implicit conversions are supplied. For constrained inputs use `Pos<N>` or
`NonNeg<N>`; unconstrained signed values are simply `N` under a `SignedFixed`
bound.

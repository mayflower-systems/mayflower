# mayf-fixed-number

Checked fixed-point traits, constrained values, and bounded expressions. The
runtime library has no dependencies, allocation, or requirement for `std`.
Import numeric types and traits directly from the crate root; implementation
modules are private. `certified` and `driver` expose the distinct contracts for
domain certificates and arithmetic implementors.

`FixedNumber` defines grid arithmetic. `SignedFixed` adds the ability to handle
negative intermediates; `UnsignedFixed` guarantees a nonnegative representation.
`NonNeg<N>`, `Pos<N>`, and `AboveOne<N>` constrain individual values. Their
private fields and checked constructors establish the predicates. Unwrapping a
refinement returns the same `N`, without a precision conversion.

Use `.bounded()` to compose refined inputs. `NonNeg` also supplies the directed
point arithmetic needed by settlement and quoting. There is no separate `Pos`
arithmetic API: an exact positive result can round to zero. Convert to `NonNeg`
when using point arithmetic, and use `Pos::try_from` if a later operation
requires strict positivity. Conversions from stronger refinements are
infallible. `NonNeg::from_unsigned` needs `UnsignedFixed` and performs no
runtime sign check; `NonNeg::from_u64` checks the driver's integer conversion
range.

## Expressions and materialized bounds

`Bounded<N>` is a nonnegative expression. It retains both outward-rounded
endpoints through addition, subtraction, multiplication, and division. Every
successful evaluation encloses its exact quantity, provided the input enclosures
and any domain certificates are sound.

`Bounds<T>` is a materialized interval with private endpoints. `Bounds::new`
checks `lower <= upper`; `Bounds::exact` constructs a singleton. Read endpoints
with `lower()` and `upper()`. Ordering alone does not establish containment of
the quantity a formula claims to represent.

Use `.bounds()` as the expression's fallible exit. It returns
`MathResult<Bounds<NonNeg<N>>>`; callers choose an endpoint at their own
quantization or settlement boundary. There is one public materialization result,
without separate terminal endpoint wrappers.

For example, the affine integral is an anchor rectangle plus a scaled delta:

```rust
use mayf_fixed_number::{Bounds, FixedNumber, MathResult, NonNeg, Pos};

fn anchored_area<N: FixedNumber>(
    anchor_y: Bounds<NonNeg<N>>,
    width: NonNeg<N>,
    delta_area: Bounds<NonNeg<N>>,
    x_scale: Pos<N>,
    y_scale: NonNeg<N>,
) -> MathResult<Bounds<NonNeg<N>>> {
    let scaled = delta_area.bounded() * y_scale / x_scale;
    (anchor_y.bounded() * width + scaled).bounds()
}
```

Raw points, refinements, and `i32`, `u32`, and `u64` literals can be lifted into
expressions. Negative inputs become sticky domain errors. Integer division in an
expression such as `NonNeg::<N>::one().bounded() / 3` encloses the fraction;
ordinary Rust integer `1 / 3` has already lost it. Raw generic points work on
the right; lift them explicitly on the left because of Rust's orphan rules.

One pending product makes `a*b/c` use one wide multiply-divide even when the
standalone product would overflow. Assignment and helper calls preserve it.
`.bounds()` evaluates a copy, leaving the original expression's pending product
intact. Other arithmetic operations and explicit `.materialize()` force it to
the grid. Thus `a*b*c/d` materializes `a*b`, then fuses the remaining product
and quotient. There is no unbounded expression tree or symbolic cancellation.

Preserve specification-defined intermediate quantizations. The application
chooses payout/charge rounding and settles conserved debits and credits once
before computing exact complements. Containment does not guarantee nearest
rounding or a universally small interval width.

## Why signed bounds remain

They serve hinged-exponential characteristic calculations. There, negative
intensity produces negative exponential inputs and alternating series terms; `exp(x)-1` and third-order remainders can be negative. Affine translation
also adds a signed offset before checking the nonnegative input domain. Prices,
areas, and the expression layer remain nonnegative.

The signed interval API provides only those calculations' operations: addition,
subtraction, negation, a product of two exact points, fused scaling by an exact
multiplier and positive divisor, and a symmetric error allowance. There is no
general signed interval multiplication/division or signed expression engine.
Required endpoint overflow is an error; error allowances never silently clip.

`bounds.try_nonnegative()` checks the endpoints without clamping.
`certified::nonnegative(bounds)` instead needs a mathematical proof that the
exact quantity is nonnegative and can remove negative rounding slack.
`certified::sub_nonnegative(a, b)` needs a proof that the exact `a >= b`.
Document these certificates at the call site and test them as domain properties.
`Bounds::intersection` similarly requires both inputs to enclose the same exact
quantity to produce a valid enclosure.

Ordinary expression subtraction requires `a.lower >= b.upper`; overlap alone
does not prove order. Division requires a strictly positive lower denominator:
exact zero gives `DivisionByZero`, while an interval touching zero gives
`Domain`. Overflow and unsupported ranges fail. Errors propagate even through
zero multiplication; existing left errors take precedence over right errors.

## Implement a driver

Implement `FixedNumber` with a uniform positive `QUANTUM`, exact checked point
construction/addition/subtraction, and two restricted primitives:

- `mul_div_floor(self, mul, div, Seal)` floors the exact wide `self*mul/div`
  once.
- `ratio_floor(u128, NonZeroU128, Seal)` supplies the directed ratio
  constructor.

Both return `Floored { value, inexact }`, where `inexact` identifies a nonzero
discarded remainder. The library derives ceiling by checked addition of one
quantum when needed. Exact operand pairs need one division/remainder calculation
for both expression endpoints. A representable quotient must survive an
unrepresentable unfused product.

These primitives are nonnegative. Signed drivers override the directed methods
and implement `SignedFixed::checked_neg`. The optional `IntegralRounding`
capability has one operation, `checked_floor_to_u64`, for whole-unit settlement.
Drivers reject negative inputs and floors above `u64::MAX`; fractional values
below the next whole unit still fit. Intermediate whole-grid values and signed
integer extraction are implementation details, not additional public methods.
`BoundedExponential` supplies the signed point enclosure for `exp(x)-1`. Drivers
must document supported ranges and fail outside them.

`checked_pow_ceil(u32)` supplies the upward-rounded integer power used by
withdrawal penalties. There is no separate downward-rounded or signed-exponent
power API without a consumer.

`driver::Seal` restricts primitive calls to the library. It does not prove a
driver's mathematical laws. The explicit formats in
[`mayf-fixed-drivers`](../mayf-fixed-drivers/README.md) implement these
contracts, including the optional signed reference driver
`mayf_fixed_drivers::reference::DashuFixed12`.

## Test the arithmetic seam

Driver tests establish grid primitive correctness. This crate's
[interval tests](tests/integration/interval_mechanics.rs) and
[expression tests](tests/integration/bounded.rs) use coarse grids and exact integer/rational
oracles to check signed scaling, outward rounding, fusion, materialization,
overflow, zero denominators, refinement predicates, and sticky errors.

Consumers test the expressions and their certificates: curve identities,
integrals, monotonicity, conservation, and settlement policy. Numeric helpers
return `MathResult`; domain boundaries choose semantic errors. Independent
end-to-end formula oracles remain valuable alongside these separate tests.

Run `cargo test -p mayf-fixed-number -p mayf-fixed-drivers -p mayf-fixed-dashu`.

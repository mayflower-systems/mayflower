# Segmented Schedule Calculator Specification

Status: Calculator layers implemented; market actions and floor-raise policy
remain separate.

## Layers

The schedule is a mathematical function of nonnegative net-issued supply. It
owns segmentation, component calculations, and contraction. It knows nothing
about circulating supply, floor redemption, option exercise, token transfers,
fees, or reserve settlement.

```text
Characteristic + Segmentation
             |
             v
compile_segmented_schedule
             |
             v
SegmentedScheduleCalculator
  y / integrate / contract_to
             |
             v
ShiftedScheduleCalculator (optional)
  y / integrate / translate
             |
             v
Market: supply accounting, actions, and economic validation
```

`mayf-monotone` retains its existing calculator contract. `mayf-characteristic`
retains its existing role: immutable neutral characteristic parameters produce
disposable monotone calculators from affine parameters. Neither contract needs
a new market operation or characteristic primitive.

The market owns the dynamic schedule state and two distinct quantities:

```rust
struct Market<C, N> {
    characteristic: C,
    segmentation: Segmentation<N>,
    net_issued_supply: NonNeg<N>,
    circulating_supply: NonNeg<N>,
    // Reserve, permissions, and other program state belong here.
}
```

This is an outline, not a required program layout or token representation.
Concrete programs choose their numeric representation and normalize token atoms
outside the mathematical crates. The characteristic's parameters remain
immutable throughout every operation.

## Coordinates and progress

All schedule coordinates are on one normalized net-issued axis `x >= 0`.
Schedule prices and integrals use compatible rate and reserve units.

Write the neutral characteristic as `c(x)`. It is increasing, smooth, and
passes through `(0, 0)`. Write the segmented schedule as `Y(x)`, with
`Y(0) = floor`. The neutral characteristic and the segmented schedule are
different functions.

The market's current net-issued supply `P` is progress along this axis. Current
price is `Y(P)`. Circulating supply `T` need not equal `P`.

There is no horizontal shift or second coordinate frame inside the core.
Its calculator takes positions on the net-issued axis directly. The optional
wrapper translates supply positions before delegating to that core. The market
maintains `P >= ramp_end` after each completed action so that the next ordinary
issuance begins on the unscaled main segment.

## Persistent segmentation

Fields are private. The following types specify roles and constraints rather
than final serialization layout:

```rust
pub struct MainReference<N: FixedNumber> {
    curve_x: NonNeg<N>,
    rate: Bounds<NonNeg<N>>,
}

pub struct Segmentation<N: FixedNumber> {
    floor: NonNeg<N>,
    ramp_scalar: AboveOne<N>,
    ramp_end: NonNeg<N>,
    ramp_width: NonNeg<N>,
    main_reference: MainReference<N>,
}
```

Use the notation:

```text
F = floor
k = ramp_scalar
E = ramp_end
W = ramp_width
A = E - W                  ramp start
(q, V) = main_reference
```

All coordinates and widths are exact grid points. Compute `A` by checked
subtraction without rounding. Do not store a redundant ramp-start field.
`W = 0` is normal and needs no artificial minimum width.

Ramp width is a client-supplied geometric witness. Continuity constrains its
mathematical value, but compilation and integration do not solve for it. A
validation layer checks its agreement with the characteristic and other state.

A structural constructor checks `W <= E`, `q <= E`, and representable derived
coordinates. Numeric refinements establish `F >= 0`, `W >= 0`, and `k > 1`.
When `E = 0`, require the canonical reference `(0, exact(F))`, establishing
`Y(0) = F` even though main begins at zero. All construction and loading paths
must perform these checks; independent unchecked setters are excluded.

### Main placement

The reference defines the main curve independently of the floor and ramp:

```text
M(x) = V + c(x) - c(q)      for x >= E
```

`V` is a certified enclosure of one particular reference value, not permission
to select a fresh value at each evaluation. Its producer certifies its truth;
ordered endpoints alone do not establish that certificate.

The reference is a representation of vertical placement. A floor raise copies
it unchanged. A contraction may replace it with a reference at the remaining
net-issued supply. Compilation never widens or rewrites persistent references.

A certified anchor works with the existing characteristic contract and avoids
repeatedly reconstructing an offset by subtracting rounded function values.
An equivalent signed vertical offset is mathematically possible, but its
representation must preserve the same precision guarantees.

Do not interpret main placement as an always-growing historical contraction
counter. If main were written `F + c(x) - D`, raising `F` would also require
raising `D` to leave main unchanged. When contraction collapses the ramp at
progress `Q`, continuity would require `D = c(Q)`, which falls toward zero as
progress is undone. An independent main reference makes those relationships
explicit without coupling every floor change to a second offset update.

## Mathematical curve

Define a floor-anchored ramp:

```text
R(x) = F + k * (c(x) - c(A))
```

The segmented schedule is:

```text
                 F          0 <= x < A
Y(x) =           R(x)       A <= x < E, when W > 0
                 M(x)       x >= E
```

Main owns the exact ramp-end point. The ramp/floor join is exact by
construction: `R(A) = F`. A continuous accepted schedule also satisfies
`R(E) = M(E)`. When `W = 0`, skip the ramp and require `M(E) = F` for continuity.

A structurally valid candidate can have an incorrect ramp/main join. The base
calculator exposes that candidate's actual piecewise values; it does not
claim that compilation proves global continuity, monotonicity, or economic
acceptance. Characteristic-dependent checks belong in a validation layer.

In particular, do not implement `MonotoneCalculator` unconditionally for an
unverified composite. A validated wrapper may provide a trait adapter later.
The component calculators already implement that trait.

## Compiler and calculator

```rust
pub fn compile_segmented_schedule<N, C>(
    characteristic: &C,
    segmentation: Segmentation<N>,
) -> ScheduleResult<SegmentedScheduleCalculator<N, C::Calculator>>
where
    N: SignedFixed,
    C: Characteristic<N>;

fn y(&self, x: NonNeg<N>)
    -> ScheduleResult<Bounds<NonNeg<N>>>;

fn integrate(&self, lo: NonNeg<N>, width: Pos<N>)
    -> ScheduleResult<Bounds<NonNeg<N>>>;

fn contract_to(&self, remaining_net_issued: NonNeg<N>)
    -> ScheduleResult<Segmentation<N>>;
```

The result retains a segmentation snapshot and disposable component
calculators. It needs neither `C: Copy` nor `C: Clone`. It does not mutate or
persist the characteristic or its calculators.

Compilation constructs affine views without evaluating their endpoints.
Component arithmetic and input-domain checks run when those views are queried;
compilation does not add a join validator or reserve policy.

The segmented query width is positive. Ramp width remains nonnegative.
Internal empty pieces are skipped. The existing component integral contract,
including its zero-width rule, stays unchanged.

### Affine compilation

Materialize these views directly on the net-issued axis:

| Component | x scale | x translation | y scale | Anchor |
| --- | --- | --- | --- | --- |
| Main | 1 | 0 | 1 | `(q, V)` |
| Ramp, when `W > 0` | 1 | 0 | `k` | `(A, exact(F))` |
| Floor | constant arithmetic | — | — | exact `F` |

All active evaluations are within the existing forward anchored domain.
At an anchor, retain its certified enclosure directly rather than subtracting
two reevaluated copies of the same function value.

An unchanged characteristic and main reference produce the same main view
across a floor raise, independent of the new floor, end, or width. Skip ramp
compilation entirely when `W = 0`; unused amplified arithmetic must not cause
an otherwise valid collapsed state to fail.

### Evaluation and integration

Select branches by exact `A` and `E`. At a nonzero ramp's start, return the exact
floor anchor; at its end, return main's enclosure.

For an integral, check that `hi = lo + width` is representable. Split `[lo, hi]`
at `A` and `E`, skip empty intersections, and integrate each active component.
The floor contributes `F * piece_width`, rounded outward. Add lower bounds
downward and upper bounds upward using the shared numeric library.

There is no division by ramp width, coordinate translation, inverse search, or
rounded boundary selection.

## Optional horizontal-shift wrapper

`ShiftedScheduleCalculator` owns a compiled core and a signed shift `h`.
It exposes `y` and `integrate` in shifted supply coordinates:

```text
visible_start = A + h
visible_end = E + h
```

Construction checks that both visible boundaries are representable and that
`visible_start >= 0`. This is a structural domain check, not a market action.
Positive shifts extend the floor tail; negative shifts consume it. Main and
ramp affine views remain unchanged.

For `x < visible_start`, return the exact floor. Otherwise delegate to core
`y(x-h)`. Integrals first handle any flat prefix directly, then delegate the
remaining positive-width interval at its translated lower endpoint. The flat
prefix may include positions with negative `x-h`; those must never reach a
characteristic calculator.

`translate(displacement)` returns a wrapper with checked `h+displacement` and
reuses its compiled core. The `inner()` accessor and consuming `into_inner()`
let callers compose this adapter without cloning characteristics or calculators.
Contraction stays on the core's net-issued axis. The wrapper performs no token
operation and owns no circulating- or net-issued-supply counter.

Thus the core is sufficient for markets without optional translation features.
A market with those features may derive `h = T-P` from its supply counts or
persist that displacement according to its own state design.

## Contraction

`contract_to(Q)` takes remaining net-issued supply, not circulating supply or
the amount sold. It computes no proceeds, fees, transfers, or market mutation.
Its exact geometry is:

```text
new_E = min(E, Q)
new_A = min(A, new_E)
new_W = new_E - new_A
```

When contraction is needed, main begins at `Q` and passes through the old
schedule's price there. The retained prefix through `Q` is unchanged, so
contraction does not alter the backing of the remaining net-issued prefix.

### Q on main: Q >= E

Return the original segmentation unchanged.

### Q in the ramp: A <= Q < E

Keep `F` and `k`, and return:

```text
main_reference = (Q, old_ramp.y(Q))
ramp_end = Q
ramp_width = Q - A
```

The retained ramp and its start are unchanged. New main is:

```text
M_new(x) = R_old(Q) + c(x) - c(Q)      for x >= Q
```

Progress is `Q`, including for a nonlinear characteristic. For an old continuous
schedule, main moves down by the constant
`(k-1) * (c(E)-c(Q))`. Subsequent buying does not restore the old main.

### Q in the floor: Q < A

Keep `F` and `k`, and return:

```text
main_reference = (Q, exact(F))
ramp_end = Q
ramp_width = 0
```

New main is:

```text
M_new(x) = F + c(x) - c(Q)            for x >= Q
```

The next issuance continues from characteristic input `Q`. Reaching the floor
collapses the ramp without resetting positive progress. At `Q = 0`, both
boundaries are zero and main is `F + c(x)`, the launch shape at the current
floor. There is no negative schedule coordinate or special origin case.

## Floor-raise candidates

A client may submit `(new_floor, new_ramp_end, new_ramp_width)` after doing any
needed search off chain. The proposal copies the characteristic, main
reference, and ramp scalar unchanged. This preserves the final segment exactly.

A geometry validator checks the intended forward/widening rules and the actual
ramp/main join. Checking the ramp at its start against the floor is tautological.
Comparing old/new composite `y` at the new end is also insufficient because both
select the preserved main. Check the signed branch residual instead:

```text
new_F + k * (c(new_E) - c(new_A)) - M(new_E)
```

Validation may reuse component compilers or materialize the same affine views
from read-only state. It does not require another core calculator operation.

The market checks `new_E <= P` and whether the candidate preserves total market
backing. Schedule geometry alone cannot decide the latter when circulating and
net-issued supply differ. The core supplies integrals for that check and does
not choose tolerances, funding, or surplus policy.

## Market accounting above the schedule

Write `P` for net-issued supply and `T` for circulating supply. Their difference
is not a field in segmentation. The following transitions illustrate the layer
boundary; authorization, fees, settlement, and rounding policies are additional
market responsibilities.

| Operation of amount n | New P | New T | Schedule action |
| --- | --- | --- | --- |
| Ordinary issuance | `P+n` | `T+n` | Unchanged |
| Ordinary redemption | `max(P-n, 0)` | `T-n` | Contract to new P |
| Redemption at floor | `P` | `T-n` | Unchanged |
| Option exercise | `P` | `T+n` | Unchanged |
| Floor raise | `P` | `T` | Check and adopt candidate |

Both redemption paths require `n <= T`. Ordinary redemption must price against
the old schedule before contraction. If `n <= P`, its curve area is over
`[P-n, P]`. If `n > P`, integrate `[0, P]` and price the extra `n-P` units at the
floor; skip the integral when `P = 0`. The schedule itself never accepts a
negative input.

### Tail capacity

To retain the original flat-tail interpretation, the market maintains:

```text
physical_tail = T - P + A >= 0
```

A floor redemption is limited by this capacity. Option exercise increases the
capacity by the amount exercised. These operations preserve `Y(P)` without
changing the schedule or progress. Ordinary redemption may instead contract
the schedule, reducing progress and changing its future continuation.

The market can enforce the capacity without a signed stored difference:

```text
P - A <= T
n <= T - (P - A)            floor-redemption limit
```

Here `P >= E >= A`, so `P-A` is nonnegative. Floor raises can create capacity by
advancing `A`, without changing net-issued or circulating supply.

### Backing and floor raising

Let `I(P)` denote the schedule's exact mathematical area over `[0, P]`. Required
market backing is:

```text
B = I(P) + F * (T-P)
  = F*T + integral[0,P] (Y(x)-F) dx
```

The second form separates floor backing for all circulating tokens from the
premium area associated with net-issued progress. The first form is useful for
comparing candidates; neither is an instruction to discard numeric bounds.
The tail-capacity invariant makes this agree with the area of the physical
circulating-supply schedule.

With supply counts unchanged, a floor raise must check the full residual:

```text
delta_B = new_I(P) - old_I(P) + (T-P) * (new_F-old_F)
```

Since main is unchanged and `new_E <= P`, the integral difference may be checked
through `new_E` instead of `P`. Preserving schedule area alone suffices when
`T = P`; otherwise the floor-backing term matters. In particular, outstanding
supply created by option exercise needs additional floor backing when the floor
rises. The market decides whether area redistribution or other funding covers
that requirement.

Zero circulating supply need not mean zero progress. After issue 100, a raise
creating enough tail, floor redemption of 50, and ordinary redemption of 50,
the market can have `T=0`, `P=50`. The contracted schedule has both boundaries
at 50 and main `F+c(x)-c(50)`. Its prefix through 50 is flat, so required backing
is zero.
The next ordinary issuance advances from input 50. Conversely, at `P=0` with
`T>0`, all required backing is `F*T`.

## Numerical and error contract

Geometry uses exact grid arithmetic. Prices, reference values, areas, and
validation residuals use certified enclosures. Copy reference enclosures whole;
never promote a midpoint or arbitrary endpoint to an exact reference value.

Compilation and contraction fail without producing a next state when geometry,
component domains, or arithmetic are unrepresentable. Translate errors into the
segmented crate's public error contract rather than exposing a driver's errors.

Repeated compilation of identical state must not mutate it or change its
mathematical meaning. A structural constructor cannot prove a reference's
provenance or a candidate's economic acceptance; validation above the core has
those obligations.

Overlapping intervals, equal enclosure objects, or a residual containing zero
do not prove equality. Verification requires the whole signed residual enclosure
to satisfy the action's conditions. Exact equality requires an exact identity
or zero residual certificate. Approximate joins must also preserve global
non-decreasingness. Tolerances, reserve safety, and cumulative error budgets
belong above the core.

## Reference example

Use `c(x)=8*x`, `k=25/16`, and initial reference `(0, exact(1))`.
Launch has `F=1`, `E=W=0`. At `P=6`, price is 49 and schedule area is 150.

An area-preserving candidate for a market with `T=P` sets `F=16`, `E=5`, `W=2`
and retains main. Exact start is 3; main is still `1+8*x`.

```text
floor [0,3]: 48
ramp  [3,5]: 57
main  [5,6]: 45
total:       150
```

From that raised state:

| Operation | P | T | Tail capacity | Price | Required backing |
| --- | ---: | ---: | ---: | ---: | ---: |
| Initial raised state | 6 | 6 | 3 | 49 | 150 |
| Redeem 3 at floor | 6 | 3 | 0 | 49 | 102 |
| Then exercise 2 options | 6 | 5 | 2 | 49 | 134 |

Separately, ordinarily redeem to `P=T=4`. Contraction yields `E=4`, `W=1`,
reference `(4, exact(28.5))`, and main `8*x-3.5`. Retained prefix area is 70.25.
Contracting the raised state to `P=T=2` instead yields `E=2`, `W=0`, reference
`(2, exact(16))`, and main `16+8*(x-2)`. Progress remains 2.

## Rewrite acceptance

The rewrite replaces the former schedule/action abstractions with this state,
compiler, core calculator, and optional horizontal-shift wrapper. Quote models,
operational mutation traits, market supply counts, reserve acceptance, fees, settlement,
and option/floor-redemption actions stay above it. No horizontal-shift field
survives in segmentation.

Use the shared numeric library and refinements. Keep the core `no_std` and free
of allocation requirements. Tests should use the existing linear driver and
independent exact integer/rational oracles to cover:

- Point values and integrals within segments and across either/both exact joins.
- Zero ramp width, positive query width, and skipped empty intersections.
- Fractional geometry, checked overflow, and invalid construction.
- Unchanged main definitions and bounds across floor-raise candidates.
- Contraction in each region, unchanged retained prefix area and current price,
  and subsequent issuance on main at the remaining characteristic input.
- A canonical zero-progress schedule at the current floor.
- Signed translations across either/both joins, tail consumption, right-shifted
  prefixes before characteristic input zero, and representable coordinates.

Market-layer tests separately cover supply transitions, floor-tail capacity,
zero circulation with positive progress, redemption beyond remaining progress,
and the extra backing term in floor-raise verification. Core tests do not
simulate option or floor-redemption operations. Avoid duplicated scalar tests
or additional characteristic fixtures merely to enlarge the test suite.

## Related specifications

- [AVM price curve and contraction](../../../docs/github-wiki/content/AVM/AVM-Specification.md)
- [Area-preserving floor raising](../../../docs/github-wiki/content/AVM/AVM-Floor-Raising.md)
- [Option execution](../../../docs/github-wiki/content/AVM/Features/Call-Options.md)
- [Characteristic contract](../mayf-characteristic/README.md)
- [Monotone calculator contract](../mayf-monotone/README.md)

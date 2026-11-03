# mayf-schedule-segmented

Floor/ramp/main calculators for Amplified Value Assets, following the
[AVM specification](../../../docs/github-wiki/content/AVM/AVM-Specification.md).
Characteristic formulas and rounding algorithms stay in their driver crates.

## Layers

- `Segmentation` stores exact geometry and an independent `MainReference`.
- `compile_segmented_schedule` constructs a disposable
  `SegmentedScheduleCalculator` on the net-issued supply axis. It provides
  `y`, `integrate`, and `contract_to`.
- `ShiftedScheduleCalculator` optionally wraps that calculator to apply a signed
  horizontal translation. It provides `y` and `integrate` in shifted supply
  coordinates, and `translate` reuses its compiled components.

The core has no horizontal-shift field. The wrapper handles coordinate placement
without knowing about token operations. Market actions, supply accounting,
quotes, fees, reserve checks, and floor-raise acceptance belong above both
calculators. Persist the neutral characteristic, segmentation, and any market
shift; derive calculators and affine views when needed.

## Curve and precision

Let the neutral characteristic be `f`, the floor `F`, the ramp end `E`, the
stored ramp width `W`, the scalar `k > 1`, and the main reference `(q, V)`.
The exact ramp start is `A = E-W`:

```text
floor: F                           for 0 <= x < A
ramp:  F + k*(f(x)-f(A))            for A <= x < E
main:  V + f(x)-f(q)                for x >= E
```

Ramp width may be zero. Main owns the exact end point; an inactive ramp is
neither compiled nor evaluated. At launch both boundaries and `q` are zero,
and `V=F`, so the schedule is `F+f(x)`.

The main reference is independent of the floor and ramp. A future floor-raise
verifier can copy it unchanged while checking a proposed floor, end, and width.
The ramp meets the floor by construction; the ramp/main join is a separate
validation obligation. Constructors check structural geometry and forward
component domains. Unverified candidates retain their actual piecewise values
and do not implement `MonotoneCalculator` unconditionally.

Coordinates use exact checked grid arithmetic. Prices and integrals return
`Bounds<NonNeg<N>>`. A bounded reference is copied whole; recompilation never
chooses a new point from it. The compiler derives affine views without probing
endpoints. Queries report component domain or arithmetic failures through
`ScheduleError`.

Integrals split at exact boundaries and combine outward-rounded component
results. Query width is `Pos<N>`; internal empty pieces are skipped. The core
requires no inverse search, numerical boundary approximation, allocation, or
floating-point conversion. Characteristics need neither `Copy` nor `Clone`.

## Contraction

`contract_to(Q)` returns new segmentation for remaining **net-issued** supply.
If `Q >= E`, it returns the original state. Otherwise, the new boundaries are:

```text
new_E = Q
new_A = min(A, Q)
new_W = new_E-new_A
```

The new main reference is `(Q, old_schedule.y(Q))`. This retains price and the
prefix area through `Q`; subsequent issuance immediately uses unscaled main
from characteristic input `Q`. Selling into the floor collapses the ramp
without resetting positive progress. At `Q=0`, main becomes `F+f(x)`.

Contraction remains in core coordinates even when a shifted wrapper is used.
Callers own the relationship between circulating supply and net-issued supply.

## Horizontal translation

A wrapper with signed shift `h` has visible boundaries `A+h` and `E+h`.
To keep the floor tail valid, construction requires `A+h >= 0` and representable
boundaries. Positive shifts extend the tail; negative shifts consume it.

Beyond the tail the wrapper delegates at input `x-h`. Within the tail it uses
constant arithmetic, including positions whose translated input would be
negative. Integration handles that flat prefix before delegating the rest.
The characteristic's affine parameters remain unchanged.

A market may add `n` to the shift when options are exercised, or subtract `n`
when tokens are redeemed at the floor, while changing circulating supply by the
same amount. These translations preserve the price at the shifted supply.
The wrapper itself performs no issuance, redemption, or balance accounting.

## Example

This state has floor 16, a ramp over `[3,5]`, and unchanged main `1+8*x`.
The optional translation consumes three units of tail:

```rust
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{AboveOne, Bounds, FixedNumber, NonNeg, Pos};
use mayf_schedule_segmented::{
    MainReference, Segmentation, ShiftedScheduleCalculator,
    compile_segmented_schedule,
};

let n = |value: u64| NonNeg::<I64F64>::from_u64(value).unwrap();
let scalar = n(25).get().checked_div_floor(n(16).get()).unwrap();
let characteristic = LinearCharacteristic::new(Pos::from_i32(8).unwrap());
let segmentation = Segmentation::new(
    n(16), n(5), n(2), AboveOne::new(scalar).unwrap(),
    MainReference::exact(n(0), n(1)),
).unwrap();
let core = compile_segmented_schedule(&characteristic, segmentation).unwrap();
assert_eq!(core.y(n(6)).unwrap(), Bounds::exact(n(49)));
assert_eq!(
    core.integrate(n(0), Pos::from_i32(6).unwrap()).unwrap(),
    Bounds::exact(n(150)),
);

let shifted = ShiftedScheduleCalculator::new(
    core, I64F64::from_i32(-3).unwrap(),
).unwrap();
assert_eq!(shifted.ramp_start(), n(0));
assert_eq!(shifted.y(n(3)).unwrap(), Bounds::exact(n(49)));
assert_eq!(
    shifted.integrate(n(0), Pos::from_i32(3).unwrap()).unwrap(),
    Bounds::exact(n(102)),
);
```

## Verification

Run `cargo test -p mayf-schedule-segmented --locked`.

Tests use independent exact integer oracles on binary, decimal, and coarse
numeric grids. They check containment and error measured in grid quanta for
prices and areas across joins, signed shifts, and contraction. Focused cases
check unchanged main placement, retained progress and prefix area, inactive
ramps, and checked arithmetic failures. Affine-view observations verify the
neutral input mapping without adding another characteristic implementation.

The [calculator specification](SPEC.md) describes the layer boundaries and the
market accounting required by future validation and floor-raise layers.

# mayf-schedule-segmented

Floor/ramp/main issuance schedules for Amplified Value Assets, following the
[AVM specification](../../../docs/wiki/avm--specification.md). The schedule works
with any implementation of `mayf_characteristic::Characteristic`; characteristic
formulas and their rounding algorithms stay in the driver crate.

`geometry` owns immutable `Segmentation` and its structural transformations.
`SegmentedSchedule` combines that geometry with a neutral characteristic and a
horizontal supply shift. It implements the `Schedule`, `FloorSchedule`, and
`RaisableFloorSchedule` contracts exported by this crate. Private calculators
derive the affine views for each calculation. Persist the characteristic,
geometry, and shift; affine transforms and calculators are disposable.

## Price and reserve

Let `f` be the neutral characteristic, `p` the floor price, `s <= e` the ramp
boundaries in curve coordinates, `k > 1` the ramp scalar, and `h` outstanding
supply issued at the floor. For total supply coordinate `x`, the curve is

```text
floor: p                                      for x <= h+s
ramp:  p + k*(f(x-h) - f(s))                   for h+s < x < h+e
main:  p + k*(f(e) - f(s)) + f(x-h) - f(e)    for x >= h+e
```

The joins are continuous. Throughout the ramp, the ramp price is below the
backward extension of main by `(k-1)*(f(e)-f(x-h))`. A zero-width ramp is valid;
launch geometry has `s = e = 0`.

Integrals split at the two boundaries, combining outward-rounded bounds from
the characteristic calculators. `required_reserve` integrates from zero to
current supply. Exact-AVA buy quotes charge the upper area bound; sell quotes
pay the lower bound. Quotes contain principal amounts before any action fees.

## State transitions

- Ordinary buys move supply along the current curve without changing geometry.
- Selling into the ramp contracts `e` to the remaining curve supply. Selling
  into the floor collapses both boundaries there. Rebuying uses this contracted
  main schedule; it does not restore the former one.
- Issuing at the floor increases `h`, extending the floor and translating the
  curve to the right. Redeeming outstanding floor-issued supply reverses that
  shift. Ordinary curve redemption cannot reduce total supply below `h`.

Geometry needs only `N: FixedNumber`; operational schedules need `N: SignedFixed`
for negative horizontal translations. The implementation uses no allocation or
floating-point conversion and accepts characteristics without `Copy` or `Clone`.
Numeric failures cross the public interface as `ScheduleError`.

## Raising the floor

A `SegmentedRaiseFloorProposal` advances `s` by a positive amount and sets the
new end to `max(old_e, new_s) + end_extension`, where the extension is
nonnegative. Geometry may widen, shorten, or collapse the ramp.

To keep prices aligned past the reshaped region, the new floor is derived from

```text
increase = k*(f(new_s)-f(old_s)) - (k-1)*(f(new_e)-f(old_e))
```

The driver evaluates these characteristic differences with anchored affine
views. The schedule takes the certified lower bound of the increase, which
must be at least one quantum. `FloorRaise::max_live_price_decrease` bounds the
possible price drop introduced by this choice. Current supply must cover the
reshaped region: the new end if it advances, otherwise the new start.

The returned schedule is a candidate. The calling action checks its required
reserve against backing and chooses an acceptable price tolerance. For the
specification's area-preserving floor raise, it selects a proposal that also
preserves required reserve. A surplus-funded floor investment may instead
increase required reserve within the available backing. Constructing a candidate
does not establish either reserve policy.

This example redistributes area exactly, raising the floor from 1 to 16 while
preserving reserve and spot price at supply 6:

```rust
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{AboveOne, Bounds, FixedNumber, NonNeg, Pos};
use mayf_schedule_segmented::{
    Schedule, Segmentation, SegmentedRaiseFloorProposal, SegmentedSchedule,
};

let n = |value: i64| NonNeg::new(I64F64::from(value)).unwrap();
let scalar = n(25).get().checked_div_floor(n(16).get()).unwrap();
let schedule = SegmentedSchedule::new(
    LinearCharacteristic::new(Pos::from_i32(8).unwrap()),
    Segmentation::new(n(1), n(0), n(0), AboveOne::new(scalar).unwrap()).unwrap(),
).unwrap();
let raised = schedule.raise_floor(
    n(6), SegmentedRaiseFloorProposal::new(Pos::from_i32(3).unwrap(), n(2)),
).unwrap();

assert_eq!(raised.new_floor, n(16));
assert_eq!(raised.next_schedule.required_reserve(n(6)).unwrap(), Bounds::exact(n(150)));
assert_eq!(schedule.required_reserve(n(6)).unwrap(), Bounds::exact(n(150)));
assert_eq!(raised.next_schedule.spot_price(n(6)).unwrap(), Bounds::exact(n(49)));
assert_eq!(schedule.spot_price(n(6)).unwrap(), Bounds::exact(n(49)));
assert_eq!(raised.max_live_price_decrease, n(0));
```

## Verification

Run `cargo test -p mayf-schedule-segmented --locked`.

Tests compare piecewise prices, integrals, contraction, and floor-raise
certificates against exact integer formulas on binary, decimal, and coarse
grids. Boundary cases cover collapsed ramps, shifts, conservative quotes,
small floor advances, and checked arithmetic failures. Successful bounds enclose
the exact values; chained rounding need not give the narrowest possible bounds,
and unrepresentable intermediates remain checked failures.

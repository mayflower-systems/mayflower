# AVM calculator

Centralized arithmetic for segmented schedules, supply accounting, trade quotes,
and floor-raise verification. A separate characteristic engine supplies only
bounded numerical evaluations of an immutable, smooth, strictly increasing
function through the origin. The caller owns storage and commits the returned
state after settlement.

The crate is `no_std`, uses no allocation, and keeps arithmetic generic over the
shared fixed-number traits.

## Numerical engine and central calculator

`CharacteristicEngine::spans` evaluates a batch of neutral characteristic
intervals. Each `SpanRequest` specifies `lo` and `width`; its `SpanResponse`
contains bounds for `f(lo)`, `f(lo + width)`, and the integral between them.
A zero-width span returns the same point bounds at both endpoints and exact
zero area. Engines return calculation errors when they cannot produce valid
bounds.

For a local curve, implement `Characteristic::y` and `Characteristic::integrate`
and wrap the implementation with `LocalCharacteristic::new`. A remote adapter
can implement `CharacteristicEngine::spans` by sending the whole batch to a
separate engine. The calculator applies the floor, ramp scalar, and offsets to
the returned bounds.

`AvmCalculator::new(ramp_scalar)` owns the shared calculation rules:

| Method                  | Result                                                     |
| ----------------------- | ---------------------------------------------------------- |
| `price`                 | Current price bounds at pricing progress                   |
| `backing`               | Required backing bounds for all circulating supply         |
| `quote_issue`           | Reserve cost bounds and next state                         |
| `quote_redeem`          | Reserve proceeds bounds and contracted next state          |
| `quote_issue_at_floor`  | Floor cost bounds and increased circulation                |
| `quote_redeem_at_floor` | Floor proceeds bounds and decreased circulation            |
| `verify_floor_raise`    | Accepted next state, proposed backing, and backing change  |

`AvmCalculator` sends the required spans for an operation in at most one batch,
including any evaluations needed for contraction or floor-raise verification.
Floor issuance and special floor redemption use only the floor price. A
transport can impose its own batch capacity.

`ScheduleCalculator` also exposes `y` and `integrate` directly for a supplied
segmentation, independently of market accounting.

```rust
use core::num::NonZeroU128;
use mayf_avm_calculator::{
    AvmCalculator, FloorRaisePolicy, FloorRaiseProposal, LocalCharacteristic,
    MarketState,
};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{AboveOne, Bounds, DirectedRounding, FixedNumber, NonNeg, Pos};

type N = I64F64;
fn n(value: u64) -> NonNeg<N> {
    NonNeg::from_u64(value).unwrap()
}
fn p(value: u64) -> Pos<N> {
    Pos::new(n(value).get()).unwrap()
}

let scalar = AboveOne::new(N::checked_from_ratio_round(
    25,
    NonZeroU128::new(16).unwrap(),
    DirectedRounding::Floor,
).unwrap()).unwrap();
let engine = LocalCharacteristic::new(LinearCharacteristic::new(p(8)));
let calculator = AvmCalculator::new(scalar);

let initial = MarketState::launch(n(1));
let issued = calculator.quote_issue(&initial, p(6), &engine).unwrap();
assert_eq!(issued.reserve, Bounds::exact(n(150)));

let raised = calculator.verify_floor_raise(
    &issued.next_state,
    FloorRaiseProposal {
        floor_increase_by: p(15),
        ramp_end: n(5),
        ramp_width: n(2),
    },
    FloorRaisePolicy::exact(),
    &engine,
).unwrap();
assert_eq!(calculator.price(&raised.next_state, &engine).unwrap(), Bounds::exact(n(49)));

let redeemed = calculator.quote_redeem_at_floor(&raised.next_state, p(3)).unwrap();
assert_eq!(redeemed.reserve, Bounds::exact(n(48)));
assert_eq!(redeemed.next_state.accounting().progress(), n(6));
assert_eq!(redeemed.next_state.accounting().circulating_supply(), n(3));
assert_eq!(calculator.price(&redeemed.next_state, &engine).unwrap(), Bounds::exact(n(49)));
```

## State and curve definition

`Segmentation` stores nonnegative floor `F`, nonnegative ramp end `E`,
nonnegative width `W`, and signed main offset `b` in private fields.
`Segmentation::new(F, E, W, b)` checks `W <= E`; ramp start `A = E-W` is
derived exactly. Zero ramp width is valid. The ramp scalar `k > 1` belongs
to the central calculator configuration.

For immutable characteristic `f`, the curve is:

```text
floor: F
main:  M(x) = f(x) + b
ramp:  R(x) = M(E) + k * (f(x) - f(E))
```

Main applies at and after `E`. Below `E`, the floor applies through `A` and the
ramp applies for `A < x < E`. The ramp joins main at `E` by construction.
Main evaluation depends only on `f` and `b`. Launch sets `b = F` and `E = W = 0`.

The central calculator composes each segment from neutral characteristic
integrals and rectangle adjustments:

```text
ramp_offset = b - (k-1) * f(E)
ramp_area = k * neutral_integral(lo, width) + ramp_offset * width
main_area = neutral_integral(lo, width) + b * width
```

These formulas also hold for nonlinear characteristics. Bounds propagate
through the arithmetic; cancellation may widen the resulting enclosure.
Restored geometry still needs to be consistent with its bound characteristic
engine. A floor-raise candidate receives a separate floor-join check before
it becomes the next market state.

`Accounting` stores progress `P` and circulating supply `T`. `MarketState::new`
checks the relationships between accounting and segmentation:

```text
P >= E
floor_tail = T - (P-A) >= 0
```

Current price is the schedule evaluated at `P`. Ordinary issuance advances both
`P` and `T`. Floor issuance changes only `T`. Special floor redemption also
changes only `T`, limited by `floor_tail`.

Ordinary redemption of `n <= T` quotes the old schedule down to
`Q = max(P-n, 0)`, with any quantity beyond `P` paid at the floor. It then updates
both counters and contracts the ramp:

```text
new_end   = min(E, Q)
new_start = min(A, Q)
new_width = new_end - new_start
```

If `Q < E`, main reattaches at the remaining progress with the corresponding
exact offset.

```text
A < Q < E:        new_b = b + (k-1) * (f(Q) - f(E))
Q < E and Q <= A: new_b = F - f(Q)
```

The next ordinary issuance begins on main. Contraction uses characteristic
values from the redemption batch. The stored offset uses an upper bound to
keep the retained schedule at or above the floor. Inside the ramp, the old
offset also bounds the new offset from above. The lower proceeds bound deducts
an upper bound on any extra retained ramp backing.

## Floor-raise verification

A proposal supplies `floor_increase_by: Pos<N>`, a new ramp end, and a new
ramp width. The positive increment represents the direction of the action in
its type. Verification checks that the joins do not move backward and the end
stays within current progress. A valid raise can narrow the ramp; width growth
is not assumed. Progress, circulation, and the stored main offset are preserved.
A proposal cannot provide a replacement main offset.

`FloorRaisePolicy::new(max_floor_join_gap, area_change_tolerance)` sets two
independent limits. The floor-join gap permits a bounded upward step from the
floor to the ramp. It cannot authorize a downward step or movement of main.
The area allowance is signed and directional, with change defined as proposed
backing minus previous backing:

| Area allowance | Permitted backing change |
| -------------- | ------------------------ |
| `+10`          | From `0` through `+10`   |
| `-10`          | From `-10` through `0`   |
| `0`            | Exactly `0`              |

The entire calculated enclosure must fit inside each permitted interval.
`FloorRaisePolicy::exact()` sets both allowances to zero.

Backing includes the physical floor tail:

```text
backing = F * floor_tail + integral(schedule, A, P-A)
```

The unchanged main tail beyond the new ramp end cancels from the backing-change
calculation. Expressed on the progress axis:

```text
backing_change = integral(new_schedule - old_schedule, 0, new_E)
               + (T-P) * floor_increase
```

The second term accounts for circulation differing from progress after floor
issuance or redemption. The verifier checks the supplied geometry and calculates
the backing change and total proposed backing using the same engine batch.
The caller chooses the directional allowance and checks that reserves cover the
proposed backing.

## Client-side floor solvers

Clients can use the `solver` module to find proposals by bisection over
characteristic values and integrals. The same search works with different
characteristic implementations.

- `solve_floor_increase` takes a positive floor increment and a maximum ramp end,
  then searches for a ramp that preserves backing within the supplied allowance.
- `solve_floor_at_ramp_end` fixes the proposed ramp end and searches for the
  area-preserving floor. The ideal maximum floor occurs at the allowed maximum
  end. On a fixed-point grid the result is a verified approximation, not a proof
  of the globally largest representable floor.

`FloorRaiseSearch` supplies an absolute `max_area_decrease`, a
`max_floor_join_gap`, and `max_iterations: NonZeroU32`. The iteration budget
limits the total characteristic batches, including final verification. Success
returns a `FloorRaiseSolution` containing the proposal and its verified quote;
the full backing-change enclosure lies in `[-max_area_decrease, 0]`.

These helpers may invoke the engine many times. They are intended for clients,
simulators, and tests; program actions still verify a supplied candidate in one
batch. Errors distinguish an invalid search range, no solution, insufficient
precision, an exhausted search budget, and arithmetic or calculator failures.
When rounding hides a root's direction, remaining grid points are checked within
the same budget. An uncertain residual never authorizes guessing its sign.

```rust
# use core::num::{NonZeroU128, NonZeroU32};
# use mayf_avm_calculator::{AvmCalculator, LocalCharacteristic, MarketState};
# use mayf_characteristic_linear_driver::LinearCharacteristic;
# use mayf_fixed_drivers::I64F64;
# use mayf_fixed_number::{AboveOne, DirectedRounding, FixedNumber, NonNeg, Pos};
# type N = I64F64;
# let n = |value| NonNeg::<N>::from_u64(value).unwrap();
# let p = |value| Pos::new(n(value).get()).unwrap();
# let scalar = AboveOne::new(N::checked_from_ratio_round(
#     25, NonZeroU128::new(16).unwrap(), DirectedRounding::Floor,
# ).unwrap()).unwrap();
# let engine = LocalCharacteristic::new(LinearCharacteristic::new(p(8)));
# let calculator = AvmCalculator::new(scalar);
# let issued = calculator.quote_issue(&MarketState::launch(n(1)), p(6), &engine).unwrap();
use mayf_avm_calculator::solver::{
    FloorRaiseSearch, solve_floor_at_ramp_end, solve_floor_increase,
};

let search = FloorRaiseSearch {
    max_iterations: NonZeroU32::new(12_000).unwrap(),
    max_floor_join_gap: n(0),
    max_area_decrease: n(0),
};
// For main = 1 + 8x, ramp scalar = 25/16, and progress = 6:
let solved = solve_floor_increase(
    &calculator, &issued.next_state, p(15), n(6), search, &engine,
).unwrap();
assert_eq!(solved.proposal.ramp_end, n(5));
assert_eq!(solved.proposal.ramp_width, n(2));

let maximum = solve_floor_at_ramp_end(
    &calculator, &issued.next_state, n(5), search, &engine,
).unwrap();
assert_eq!(maximum.proposal.floor_increase_by, p(15));
assert_eq!(maximum.quote.next_state, solved.quote.next_state);
```

## Using quotes and verified state

Quotes return bounds for gross reserve cost or proceeds in normalized units.
Use the upper bound for an issuance charge and the lower bound for a redemption
payout. The caller applies fees and token-unit rounding and commits the next
state together with settlement.

Bind each market to its characteristic and ramp scalar for its lifetime.
Checked constructors establish structural geometry and supply relationships.
The caller supplies authenticated state consistent with that characteristic,
establishes reserve coverage and custody, and relies on its selected engine to
enclose the true values and integrals.

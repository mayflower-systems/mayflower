# AVM calculator

Centralized arithmetic for segmented schedules, supply accounting, trade quotes,
and floor-raise verification. A separate characteristic engine supplies only
bounded numerical evaluations of an immutable, increasing function through the
origin. The caller owns storage and commits the returned state after settlement.

The crate is `no_std`, uses no allocation, and keeps arithmetic generic over the
shared fixed-number traits.

## Numerical engine and central calculator

`CharacteristicEngine::spans` evaluates a batch of neutral characteristic
intervals. Each `SpanRequest` specifies `lo` and `width`; its `SpanResponse`
contains bounds for `f(lo)`, `f(lo + width)`, and the integral between them.
A zero-width span returns the point value at both endpoints and exact zero area.
Engines return calculation errors when they cannot produce valid bounds.

`LocalCharacteristic::new(characteristic)` evaluates each span using the neutral
`Characteristic::y` and `Characteristic::integrate` methods. A remote adapter
can encode the complete batch in one invocation. The central calculator applies
the floor, ramp scalar, and offsets to the returned bounds.

`AvmCalculator::new(ramp_scalar)` owns the shared calculation rules:

| Method                  | Result                                                 |
| ----------------------- | ------------------------------------------------------ |
| `price`                 | Current price bounds at net-issued progress            |
| `backing`               | Required backing bounds for all circulating supply     |
| `quote_issue`           | Reserve cost bounds and next state                     |
| `quote_redeem`          | Reserve proceeds bounds and contracted next state      |
| `quote_issue_at_floor`  | Floor cost bounds and increased circulation            |
| `quote_redeem_at_floor` | Floor proceeds bounds and decreased circulation        |
| `verify_floor_raise`    | Accepted next state and bounded verification residuals |

Operations requiring characteristic arithmetic prepare an array of the needed
requests before calling `spans` once. Prices, issuance, and backing need at most
two spans. A redemption or arbitrary integral beginning inside the ramp and
ending on main needs three: the two integration intervals and the ramp-start
point. Floor raises need at most two spans, with unchanged or identical boundary
movements omitted or reused. Floor issuance, floor redemption, and geometric
contraction require no engine call. There is no global batch-capacity setting
or heap allocation; the cost of each characteristic evaluation depends on the
engine.

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
        floor: n(16),
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

`Segmentation` has three private fields: nonnegative floor `F`, nonnegative ramp
end `E`, and nonnegative width `W`. Construction checks `W <= E`; ramp start
`A = E-W` is derived exactly. Zero ramp width is valid. The ramp scalar `k > 1`
belongs to the central calculator configuration.

For immutable characteristic `f`, the curve is:

```text
floor: F
ramp:  R(x) = F + k * (f(x) - f(A))
main:  M(x) = R(E) + f(x) - f(E)
```

Floor applies before `A`, ramp from `A` to `E`, and main from `E` onward. Both
joins are continuous by construction. The central calculator composes each
segment from neutral characteristic integrals and rectangle adjustments:

```text
ramp_offset = F - k * f(A)
main_offset = F + (k-1) * f(E) - k * f(A)

ramp_area = k * neutral_integral(lo, width) + ramp_offset * width
main_area = neutral_integral(lo, width) + main_offset * width
```

These formulas hold for nonlinear characteristics too. Derived offsets are
temporary signed bounds, never independently stored parameters. Bounds are
propagated through the arithmetic; cancellation may widen the resulting
enclosure.

`Accounting` stores progress `P` and circulating supply `T`. `MarketState::new`
checks the relationships between accounting and segmentation:

```text
P >= E
floor_tail = T - (P-A) >= 0
```

Current price is the schedule evaluated at `P`. Ordinary issuance advances both
`P` and `T`. Floor issuance changes only `T`. Special floor redemption also
changes only `T`, limited by `floor_tail`. These optional operations do not
evaluate or mutate characteristic parameters.

Ordinary redemption of `n <= T` quotes the old schedule down to
`Q = max(P-n, 0)`, with any quantity beyond `P` paid at the floor. It then updates
both counters and contracts segmentation using only exact geometry:

```text
new_end   = min(E, Q)
new_start = min(A, Q)
new_width = new_end - new_start
```

The retained prefix is unchanged, and the next ordinary issuance begins on
main. This contraction needs no characteristic evaluation or inverse. The
characteristic input remains the actual progress coordinate for nonlinear
engines as well.

## Floor-raise verification

A proposal supplies a new floor, ramp end, and ramp width. Verification checks
that the floor increases, the joins do not move backward, and the end stays
within current progress. Progress and circulation remain unchanged. A valid
raise can narrow the ramp; width growth is not assumed.

Both the old and proposed main follow the same unscaled characteristic. Their
offset difference measures the price change throughout their common main domain.
Backing includes the physical floor tail:

```text
backing = F * floor_tail + integral(schedule, A, P-A)
```

Verification computes changes directly from the spans between the old and new
ramp starts and between the old and new ramp ends. For either boundary movement
from `lo` to `hi`, at unchanged progress `P`:

```text
height_change = f(hi) - f(lo)
G(lo, hi) = neutral_integral(lo, hi-lo) - f(lo)*(hi-lo)
           + (P-hi)*height_change

main_change = floor_increase + (k-1)*end_height_change - k*start_height_change
backing_change = T*floor_increase + (k-1)*G(old_E, new_E) - k*G(old_A, new_A)
```

The shared characteristic area cancels, so verification does not recalculate
both complete backing amounts. These expressions retain any permitted main
price change and account for circulation `T` differing from progress `P`.

`FloorRaisePolicy::new(max_price_change, area_change_tolerance)` checks the
entire main-offset change against the symmetric price allowance. The area
allowance is signed and directional, with change defined as proposed backing
minus previous backing:

| Area allowance | Permitted backing change |
| -------------- | ------------------------ |
| `+10`          | From `0` through `+10`   |
| `-10`          | From `-10` through `0`   |
| `0`            | Exactly `0`              |

The entire backing-change enclosure must lie inside the permitted interval.
`FloorRaisePolicy::exact()` sets both allowances to zero. Approximate area
preservation uses a small directional allowance; a raise funded from surplus
can use a positive allowance. Both use the same verifier, which returns the
measured main and backing changes with the next state.

The higher-level program chooses policy and establishes that any permitted
increase is funded. This arithmetic crate does not inspect reserve accounts or
authorize spending surplus. It verifies a supplied candidate without searching
for geometry or inverting the characteristic.

## Storage and settlement boundary

Quotes use normalized quantities and return bounds for gross reserve cost or
proceeds. The host chooses the upper bound for a charge and the lower bound for
a payout, applies its normalization and integer rounding, and settles fees,
transfers, and state persistence atomically. Conservative bounds need not give
the nearest possible whole-unit result.

`mayf-avm-prog` wraps this calculator with pure program actions, reserve and
position accounting, and fees. It remains generic over the number type so the
same actions can run in a desktop simulator. Semantic operations such as
`raise_floor_from_surplus` belong in that layer.

A future `mayf-avm-prog-sol` will select concrete numeric types, normalize token
units, encode CPI requests, check authorization, map program errors, and perform
transfers and persistence. Only the numerical span batch crosses the
characteristic program boundary; quoting, validation, and next-state
construction stay central.

Restored state must remain bound to the same characteristic parameters and ramp
scalar. Checked constructors establish geometry and supply relationships; they
do not establish account authorization, actual reserve balances, collateral
health, or the truth of bounds supplied by an untrusted engine.

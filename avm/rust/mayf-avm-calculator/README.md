# AVM calculator

Stateless quotes and next-state receipts for continuous segmented schedules.
An engine supplies characteristic arithmetic; this crate supplies segmentation,
progress accounting, contraction, and floor-raise verification. The caller owns
the stored state and decides when to commit a receipt.

The crate is `no_std`, uses no allocation, and keeps arithmetic generic over the
shared fixed-number traits.

## Engine contract

To support a new characteristic:

1. Implement `MonotoneCalculator::y` and `integrate` for its affine views,
   returning certified bounds or a calculation error.
2. Implement `Characteristic::calculator` to construct those views from
   `AffineTransform`. The neutral characteristic is increasing and passes
   through `(0, 0)`.
3. Wrap the characteristic in `CharacteristicEngine` with a fixed ramp scalar.

The wrapper implements the object-safe `AvmCalculator` contract automatically:

| Method | Result |
| --- | --- |
| `price` | Current price bounds at net-issued progress |
| `backing` | Required backing bounds for all circulating supply |
| `quote_issue` | Reserve cost bounds and next state |
| `quote_redeem` | Reserve proceeds bounds and contracted next state |
| `quote_issue_at_floor` | Floor cost bounds and increased circulation |
| `quote_redeem_at_floor` | Floor proceeds bounds and decreased circulation |
| `verify_floor_raise` | Accepted next state and bounded verification residuals |

Engine authors reuse these operations without implementing another version of
the market rules. No persistent state or token balance is held inside the
engine.

```rust
use core::num::NonZeroU128;
use mayf_avm_calculator::{
    AvmCalculator, CharacteristicEngine, FloorRaisePolicy, FloorRaiseProposal,
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
let engine = CharacteristicEngine::new(LinearCharacteristic::new(p(8)), scalar);
let calculator: &dyn AvmCalculator<N> = &engine;

let initial = MarketState::launch(p(1));
let issued = calculator.quote_issue(&initial, p(6)).unwrap();
assert_eq!(issued.reserve, Bounds::exact(n(150)));

let raised = calculator.verify_floor_raise(
    &issued.next_state,
    FloorRaiseProposal {
        floor: p(16),
        ramp_end: n(5),
        ramp_width: n(2),
    },
    FloorRaisePolicy::exact(),
).unwrap();
assert_eq!(calculator.price(&raised.next_state).unwrap(), Bounds::exact(n(49)));

let redeemed = calculator.quote_redeem_at_floor(&raised.next_state, p(3)).unwrap();
assert_eq!(redeemed.reserve, Bounds::exact(n(48)));
assert_eq!(redeemed.next_state.accounting().progress(), n(6));
assert_eq!(redeemed.next_state.accounting().circulating_supply(), n(3));
assert_eq!(calculator.price(&redeemed.next_state).unwrap(), Bounds::exact(n(49)));
```

## State and curve definition

`Segmentation` has three private fields: positive floor `F`, nonnegative ramp
end `E`, and nonnegative width `W`. Construction checks `W <= E`; ramp start
`A = E-W` is derived exactly. Zero ramp width is valid. The ramp scalar `k > 1`
belongs to the engine configuration.

For immutable characteristic `f`, the curve is:

```text
floor: F
ramp:  R(x) = F + k * (f(x) - f(A))
main:  M(x) = R(E) + f(x) - f(E)
```

Floor applies before `A`, ramp from `A` to `E`, and main from `E` onward. Both
joins are continuous by construction. Compilation derives a bounded ramp-end
price and anchors main there. That price is disposable calculated data; it is
not another stored parameter. A zero-width ramp is skipped.

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
that the floor increases, the joins do not move backward, the ramp does not
narrow, and the end stays within current progress. Progress and circulation
remain unchanged.

Both the old and proposed main follow the same unscaled characteristic. Their
price difference at the new end is therefore their vertical displacement over
the common main domain. Backing includes the physical floor tail:

```text
backing = F * floor_tail + integral(schedule, A, P-A)
```

`FloorRaisePolicy::new(max_price_change, max_reserve_surplus)` accepts only when
the entire signed main-price residual lies within the configured symmetric
price allowance, and the entire signed backing residual lies in
`[-max_reserve_surplus, 0]`. The residuals are proposed minus old values.
`FloorRaisePolicy::exact()` sets both allowances to zero.

These checks never authorize an unproven increase in required backing. When
rounding prevents certifying exact area equality, a client can choose a nearby
proposal leaving a small, permitted reserve surplus. Merely overlapping bounds
does not establish acceptance. The host supplies the policy from trusted market
configuration; it is not a caller-selected slippage allowance.

## Storage and settlement boundary

Quotes use normalized quantities and return bounds for gross reserve cost or
proceeds. The host chooses the upper bound for a charge and the lower bound for
a payout, applies its normalization and integer rounding, and settles fees,
transfers, and state persistence atomically. Conservative bounds need not give
the nearest possible whole-unit result.

This crate does not choose a `u64` wire encoding, normalization metadata, CPI
transport, fees, or custody rules. A Solana adapter can expose the same quote
operations through a versioned request/response contract and keep the returned
state in accounts owned by the main program. Its compute budget still depends
on the selected characteristic implementation.

Restored state must remain bound to the same characteristic parameters and ramp
scalar. Checked constructors establish geometry and supply relationships; they
do not establish account authorization, actual reserve balances, collateral
health, or the truth of bounds supplied by an untrusted engine.

## Comparing the designs

`mayf-schedule-segmented` keeps an independent main reference and exposes an
optional horizontal-translation calculator. `mayf-avm-prog` remains its separate
program prototype.

This crate derives main placement from the ramp endpoint, represents horizontal
effects through separate progress and circulation, and provides pure quote
operations with caller-owned next-state receipts. The existing crates are kept
alongside it for comparison.

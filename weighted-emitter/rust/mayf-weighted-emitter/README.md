# mayf-weighted-emitter

A pure `no_std` state machine that divides a declining emission stream among
weighted gauges. Actions consume model drafts and return the next records plus
whole token amounts. There is no Solana, storage, custody, referral logic, or
token minting in this crate.

## Accounting

An `Emitter` owns the schedule, total gauge weight, and a precise cumulative
emission-per-vote index. Each `Gauge` owns its weight, consumed index, and
cumulative whole-unit allocation. Gauge weights are whole `u64` votes.

Advancing the emitter adds `floor(interval_budget / total_weight)` to its index.
Settling a gauge allocates `floor(index_growth * gauge_votes)` whole token base
units and advances its entire checkpoint. Changing votes first settles the
gauge at its old weight. Other gauges retain their pending index growth.
Creating a gauge starts at the current index. During intervals without votes,
emission is discarded while the rate continues to decay; later voters receive
no backpay.

There are no dust carries, residual accounts, or interval-closing operations.
Every rounding operation rounds down. Gauge settlement discards less than one
token **base unit**, not one displayed token: for a six-decimal token, a base
unit is 0.000001 tokens. More frequent settlements can pay less; repeatedly
settling sub-unit allocations can discard them entirely. The host chooses who
can settle and when. This is intentional truncation, not frequency-independent
accounting.

Use a wide driver such as `mayf_fixed_drivers::U192F64`. Creation requires a grid
at least as fine as 2^-64 and enough range for `u64` votes. Index division loses
less than `total_weight * quantum` per checkpoint, below one base unit on that
grid. Schedule arithmetic also rounds down. No floating point is used.
Individual allocations and both cumulative materialized totals are checked
`u64` values; overflowing any total or numeric intermediate rejects the action.

## Cheap compounding decay

The annual decay coefficient `d` approximates continuous decay without a
transcendental function. With `Y = 31,536,000` seconds (365 days), the per-second
retention factor is:

```text
q = floor_to_numeric_grid(Y / (Y + d))
next_rate = floor_to_numeric_grid(rate * q^elapsed_seconds)
interval_budget = floor_to_numeric_grid(rate * (q + q^2 + ... + q^elapsed_seconds))
```

A second pays its ending rate. `d = 0` gives a constant rate; `d = 0.1`
approximately retains 90.48% after one year. This recurrence defines the
implemented schedule: it is an approximation, not a certified bound on the
ideal continuous exponential. All calculations conservatively bound the
recurrence itself. Fixed-point rounding also occurs inside the calculation.

Binary interval composition computes the rate and geometric sum together in
at most 64 iterations, even across the full `u64` time range. It needs only
checked addition, multiplication and division. The rate never increases.
Updating decay first accrues the old schedule, then applies the new coefficient
to the current rate; it cannot reset the rate or rewrite earlier allocations.

## Actions and host responsibilities

| Action                   | Result                                                       |
| ------------------------ | ------------------------------------------------------------ |
| `create_emitter`         | Empty emitter with a starting rate, decay and timestamp      |
| `create_gauge`           | Zero-weight gauge checkpointed at the current index          |
| `set_gauge_votes`        | Old-weight settlement followed by an aggregate weight change |
| `emit_to_gauge`          | Whole-unit allocation and updated cumulative totals          |
| `set_emitter_decay_rate` | Prospective decay change after old-schedule accrual          |

Apply every returned record and allocation effect atomically, or none of them.
Authenticate canonical records, unique IDs, the clock, administrative changes,
and backed voting weights. `set_gauge_votes` accepts an authorized aggregate
weight; individual voters and their token custody belong to the host. Model
linkage, checkpoint consistency, monotone time, and arithmetic are checked here.
The library cannot reconstruct the sum of all gauges from one supplied gauge.

`GaugeReceipt::emitted` is already included in both returned `total_emitted`
counters. It is a new allocation, not an additional cumulative amount or an
instruction to mint. A future consumer can use the gauge's cumulative total and
timestamp, with an authenticated source binding and a persisted consumed total.
Do not assign the same entire stream to multiple reward consumers.

```rust
use mayf_fixed_drivers::U192F64;
use mayf_fixed_number::NonNeg;
use mayf_weighted_emitter::*;

let emitter = create_emitter(CreateEmitterArgs {
    id: 1u64,
    emission_rate: NonNeg::from_unsigned(U192F64::from_u64(10)),
    annual_decay_rate: NonNeg::zero(),
    at: 0,
})?.new_emitter;
let created = create_gauge(CreateGaugeArgs { emitter, id: 2, at: 0 })?;
let weighted = set_gauge_votes(SetGaugeVotesArgs {
    emitter: created.emitter_next,
    gauge: created.new_gauge,
    votes: 1,
    at: 0,
})?;
let settled = emit_to_gauge(EmitToGaugeArgs {
    emitter: weighted.emitter_next,
    gauge: weighted.gauge_next,
    at: 3,
})?;
assert_eq!(settled.emitted, 30);
assert_eq!(settled.gauge_next.total_emitted, 30);
# Ok::<(), WeightedEmitterError>(())
```

Run the ordinary Rust tests with `cargo test -p mayf-weighted-emitter --locked`.
They cover allocation, prospective changes, intentional truncation, invalid
records, overflow, and independent integer oracles for the decay approximation.

# referral-tree

`referral-tree` holds the numeric rules of a referral network. It turns
integer deposits into points and divides each deposit's points between the
depositor and its upline. It then accumulates those points over time as
point-time. A member can also pass part of its point-time down to its direct
children.

The rules reward recruiting members who deposit. They do not create value.
Total points always equal total deposits, so the rules only decide who is
credited with each deposit's points. Deposited principal never moves.

The crate is a pure `no_std` library. It has no identities, storage, clock,
assets, or reward schedule. The caller picks a fixed-point type that implements
`mayf_fixed_number::FixedNumber`, and an integer clock unit. Every action takes
the current state and returns replacement state.

The Mayflower consumer, `mayf-referral`, fixes its numeric policy: compact
`U64F64` rates widen exactly onto the `U192F64` accounting grid. It exposes
concrete tree configuration and state types while this library remains generic
for other consumers and arithmetic implementations.

## Terms

- **Tree and depth.** Members form a forest of permanent parent-child links. A
  top member has depth 0, its children depth 1, and so on.
- **Upline and downline.** A member's upline is its parent, its parent's
  parent, and so on up to a top member. Its downline is everyone below it: its
  children, their children, and so on. Neither includes the member itself.
- **Maximum depth `m`.** The deepest permitted depth. `m = 2` permits three
  levels: depths 0, 1, and 2.
- **Deposit `p`.** A member's own integer deposit. One deposit unit is one point
  before overrides.
- **Source.** The member whose deposit is being divided.
- **Override rate `q`.** The fraction of a source's deposit credited to _each_
  member of its upline. Each such credit, `q * p`, is an override.
- **Core points `b`.** What a member keeps from its own deposit, plus the
  overrides from its whole downline.
- **Sharing `μ`.** The fraction of its earnings a member passes to its direct
  children. Also called _mutualization_.
- **Point-time.** Points multiplied by elapsed clock ticks, such as
  point-seconds.
- **Lane.** One independent set of deposits over the shared tree, such as one
  reward channel. A member has a separate `Node` in each lane, with its own
  deposit, sharing fraction, and history. A tick must mean the same unit
  throughout a lane.
- **Grid and quantum `ε`.** The values the chosen fixed-point type can
  represent, and the gap between neighboring values.

## Core points

A forest fixes `m` and `q` for its whole life. They must satisfy:

```text
0 <= q < 1        m * q < 1
```

A source at depth `d` has `d` members in its upline. It credits an override
of `q * p` to each of them and keeps the rest, `p - d * (q * p)`. The second
bound keeps that rest positive at every depth. Each override is taken from the
source's original deposit, not from what a nearer upline member left. So a
grandparent's override does not depend on the parent's deposit.

Add up a member's kept part and the overrides from its downline. A member at
depth `d` whose downline deposits `D` in total has:

```text
b = (1 - d*q) * p + q * D
```

Example: `m = 2` and `q = 10%`. Alice is Bob's parent, and Bob is Charlie's.
Each deposits 100.

| Member  | Depth | Kept from own deposit | Overrides from downline | Core points |
| ------- | ----: | --------------------: | ----------------------: | ----------: |
| Alice   |     0 |                   100 |                 10 + 10 |         120 |
| Bob     |     1 |                    90 |                      10 |         100 |
| Charlie |     2 |                    80 |                       0 |          80 |
| Total   |       |                       |                         |         300 |

Three properties follow. Each one is a design goal:

- **Conservation.** A source's kept part plus its overrides equals its deposit.
  So the core points of a forest sum to its total deposits. Referral credit
  divides a fixed budget; it never grows it.
- **Rewarding real recruitment.** A member with no deposit still earns
  overrides from downline members who deposit. Recruiting empty accounts earns
  nothing.
- **No gain from splitting.** Take a subtree whose top sits at depth `d`, with
  total deposits `S`. Its combined core points are `(1 - d*q) * S`, wherever
  `S` sits inside it. So moving existing capital among accounts one person
  controls, under an unchanged outside upline, cannot raise their combined
  core points. The guarantee does not cover voluntary sharing, buying a shallower
  position, or placing a controlled account above a future outside recruit.

## Sharing

Each member chooses a sharing fraction `μ` from 0 to 1. It starts at 0. A
member's available earnings `A` for an interval are its core earnings plus what
its parent shared with it. Let `Z` be the total deposits of its direct children.
In ideal arithmetic:

```text
gift to child c = μ * A * p_c / Z
member keeps    = (1 - μ) * A
```

- Children divide a gift by their own deposits. Their own downlines and their
  number do not count.
- A child can share what it receives onward. Shared amounts never pay overrides
  a second time.
- If `Z = 0`, the member keeps all of `A`. A child with no deposit receives
  nothing from its parent.
- A change to a deposit or to `μ` applies from the moment of the change. Earlier
  intervals keep the values that applied then. A new child gets nothing from
  intervals before it joined.

Sharing lets a parent make its position attractive to recruits. It moves
existing earnings down the tree and creates nothing.

## Point-time and lazy settlement

A member can have any number of children, and the library never loops over
them. Each action touches only one path from a top member down to a source.
Its work is O(`m`), however wide the tree grows. This bound matters where
computation is metered, as in a blockchain transaction.

Sharing therefore uses a cumulative index. Each node keeps `U`, the total
point-time it has shared per unit of direct-child deposit. When a parent
shares, its children are not touched. A child is owed
`p_child * (U_now - U_seen)`, where `U_seen` is the parent's index at the
child's last settlement. The child collects this when it next settles. This is
the reward-per-share pattern of staking contracts.

Settling a node at `time`, after its parent has settled at the same time,
computes:

```text
available = core_points * (time - last_time)
          + deposit * (parent.shared_point_time_index - last_parent_point_time_index)

if child_deposits > 0:
    index_increment = floor_ε(available * sharing / child_deposits)
else:
    index_increment = 0

shared_point_time_index      += index_increment
point_time                   += available - child_deposits * index_increment
last_time                     = time
last_parent_point_time_index  = parent.shared_point_time_index
```

The `Node` field docs define each field. These rules follow:

- **Parents settle first.** `checkpoint_path` settles a whole path, top first.
- **Stored point-time is stale.** `point_time` covers time up to `last_time`
  only. To read a current value, run `checkpoint_path` at the current time.
  Reading the result does not require persisting it.
- **Changes settle old history first.** `set_deposit` and `set_sharing` settle
  the path under the old values, then apply the new value.
- **A deposit change touches only its path.** It updates the
  `downline_deposits` of every upline member and the parent's `child_deposits`. Siblings need no
  update. Their deposits did not change, and their rights wait in the parent's
  index.
- **New members start clean.** `initialize_node` starts a node with zero
  deposit, at the current time and its settled parent's current index. It
  cannot claim history from before it joined.
- **History is permanent.** Emptying a deposit keeps the node's point-time and
  index history. `point_time` and `shared_point_time_index` never decrease, and
  nothing withdraws point-time.
- **The whole forest conserves.** Retained point-time plus unsettled rights
  equals the integral of total deposits over time.

## Rounding

The caller chooses the grid: its precision and its range. The library calls
only `FixedNumber` methods, each with an explicit rounding direction. It never
reads raw bits and never carries remainders between calls.

The library's only dependency is `mayf-fixed-number`. Consumers choose `N` at
build time, including their own trait implementation. Tests depend only on
`mayf-fixed-drivers` with its optional `reference` feature: the public binary
drivers, `I64Scaled<10>`, and `U256Scaled<10>`/`U256Scaled<1_000_000_000_000>`
cover signed, unsigned, coarse, and wide decimal grids. The independent history
oracle uses `reference::UBig` for exact integer arithmetic, and
`reference::DashuFixed12` supplies a separate signed decimal implementation.
No fixed-point implementation lives in the referral tests.

- **Overrides are exact once the rate is represented.** 10% is exact on a
  decimal grid but not on a binary one. When converting an outside ratio to
  `q`, round up. Upline members then never receive less than the stated rate,
  and a small positive rate cannot round to zero and exempt small deposits.
  Check the rounded value with `validate_config`. After that, `q * p` is a grid
  value times an integer, which is exact, and the kept part is found by
  subtraction. Core points conserve deposits exactly.
- **Base point-time is exact.** `core_points * ticks` is also a grid value times
  an integer. It does not depend on how often nodes settle.
- **Sharing rounds toward the parent.** The index is rounded down in one fused
  multiply-divide. The member keeps `available - Z * index_increment`, found by
  subtraction, so kept plus shared equals available exactly. The member keeps
  whatever is too small to index: less than `Z * ε` per settlement, even at 100%
  sharing. This parent bias is deliberate. The kept amount is not carried
  forward, so settling more often can lower realized sharing. The effect is
  largest on coarse grids.
- **Point-time needs a wide type.** A tree's deposits fit in `u64`, and so does
  the clock. So in state built by these actions, no point-time value, sharing
  index, or intermediate sum can exceed `u64::MAX * u64::MAX`, about 2^128. The point-time actions check that
  the chosen type holds this value before they settle anything, and otherwise
  return `Error::RangeTooSmall`. A narrow type therefore fails on first use. It
  cannot overflow later and lock every path through a node, withdrawals
  included. `U192F64` qualifies. `U64F64` and `DashuFixed12` do not, but
  `effective_points` and `share_points` accept them, because those pure
  calculators have no range requirement.
- **Deposits must fit `u64`.** A node's deposit plus its downline's deposits
  must fit `u64`, or the action returns `Error::Overflow`.

## Point-time is not a reward

Point-time measures participation. It does not by itself set a payout when the
reward rate changes over time. Take a lane with reward rate `e(t)` and total
deposits `P(t) > 0`. Its reward per deposit unit is:

```text
J(t) = ∫ e(t) / P(t) dt
```

Two members with equal point-time can earn different rewards if they were
active at different rates. Neither a point-time total nor an average rate can
recover that.

A reward composition therefore applies the same rules to `J` instead of to
ticks. A member's available reward for an interval is
`core_points * ΔJ + deposit * Δ(parent's reward index)`, split with
`share_points`. This is why `effective_points` and `share_points` are public.
Mayflower Reserve composes the crate this way. Reward balances, collection, and
assets belong to the composition, not to this crate.

## Using the crate

The caller owns storage. In this example, a `Vec` ordered top to source holds
the path Alice → Bob → Charlie. Point-time needs a wide type, so the example
uses `U192F64`, a binary grid. 10% is not exact on a binary grid, so the
example uses 12.5%, which is.

```rust
use core::num::NonZeroU128;
use mayf_fixed_drivers::U192F64;
use referral_tree::{
    CheckpointPathArgs, Config, DirectedRounding, FixedNumber, Node, SetDepositArgs,
    checkpoint_path, effective_points, set_deposit,
};

fn whole(n: i128) -> U192F64 {
    U192F64::checked_from_i128(n).unwrap()
}

let config = Config {
    max_depth: 2,
    // Round an outside ratio up, so upline members never get less than the
    // stated rate. 1/8 is exact here, so rounding changes nothing.
    override_rate: U192F64::checked_from_ratio_round(
        1, NonZeroU128::new(8).unwrap(), DirectedRounding::Ceil,
    ).unwrap(),
};

// Each member deposits 100 at tick 0. A deposit changes the source's path only.
let mut path = vec![Node::<U192F64>::default(); 3];
for source in 0..3 {
    let args = SetDepositArgs { config, time: 0, new_deposit: 100 };
    let next = set_deposit(&path[..=source], args).unwrap().next_path;
    path[..=source].copy_from_slice(&next);
}

let points: Vec<_> = path
    .iter()
    .enumerate()
    .map(|(depth, node)| {
        effective_points(config, depth as u8, node.deposit, node.downline_deposits).unwrap()
    })
    .collect();
assert_eq!(points, [whole(125), whole(100), whole(75)]);

// Settle ten ticks later. Point-time is core points times elapsed ticks.
let path = checkpoint_path(&path, CheckpointPathArgs { config, time: 10 })
    .unwrap()
    .next_path;
assert_eq!(path[0].point_time, whole(1250));
assert_eq!(path[2].point_time, whole(750));
```

The caller must:

- **Authenticate the path.** Validation rejects impossible numbers, such as a
  child holding more than its parent's totals. Numbers alone cannot prove that a
  supplied path matches the stored tree.
- **Commit every returned node atomically.** Actions never mutate their inputs,
  so after an error the stored state is unchanged.
- **Supply a clock that never goes backwards.**
- **Keep `Config` fixed** for the life of the forest.

## Verification

```sh
cargo test -p referral-tree
cargo run -p referral-tree --example testbench
```

Every test calls the public API. Reference calculations are independent test
code.

- `tests/integration/protocol.rs` checks the worked examples, recursive
  sharing, parent rounding bias, onboarding, dormant siblings, and input
  validation. It uses a deliberately coarse 0.1-unit grid with a 256-bit range.
- `tests/integration/allocation_contract.rs` checks conservation, the
  splitting guarantee, and the cases outside that guarantee.
- `tests/integration/numeric_drivers.rs` runs the pure calculators on every
  driver, including signed and 64-bit ones, and point-time on the wide
  drivers. It checks that narrow types are rejected for point-time, and that
  the largest possible point-time settles exactly.
- `tests/integration/lazy_history.rs` compares thousands of deposit, sharing,
  and settlement steps against an arbitrary-precision reference, on binary,
  decimal, and coarse grids. It checks exact conservation of total point-time.
- `examples/testbench.rs` prints Alice, Bob, and Charlie's point-hours before
  and after Bob starts sharing 50%.

## Related crate

The `mayf-referral` crate supplies the membership records: identities,
permanent paths, admission offers, and per-scope deposits.

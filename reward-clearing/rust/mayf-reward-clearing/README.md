# mayf-reward-clearing

The pure Rust algorithm for the [Reward Clearing auction](../../README.md).

## Calling the algorithm

[Actions](src/actions/) take accounting state and explicit inputs and return
replacement state with a receipt. Each action has its own file, and
[`actions.rs`](src/actions.rs) lists their public items. The public model
consists of an epoch schedule, immutable terms, active and finalized epochs,
position totals, and lots. It has no account identities, authorities, tenant
creation, fee policy, lot storage, or account closure operations.

A shared `EpochSchedule` derives aligned windows for channels that synchronize
epochs. Starting an epoch fixes its terms. Deposits issue weighted shares;
withdrawals surrender the newest lot's shares and split principal between a
return and a penalty. Settlement freezes the reward/share ratio, and claiming
reports a participant's earned whole reward. `remaining_reward` reports the
tracked remainder after every share has claimed.

The [auction example](examples/auction.rs) runs two deposits, funding,
settlement, and claims using only these values. Run it with
`cargo run -p mayf-reward-clearing --example auction`.

```text
host selects and authenticates the relevant accounting state
    → calculate a clearing receipt
    → host atomically applies its amounts and replacement state
```

Receipts contain no transfer endpoints or signing instructions. A receipt is a
calculation, not proof that a balance changed. The host must prevent replay and
apply the replacement state together with the resulting balance movements.

## Data and host responsibilities

[`state`](src/state.rs) provides validated terms, schedule windows, active and
finalized accounting, positions, lots, and plain accounting snapshots.
Validation checks the algorithm's invariants: time bounds, valid weights,
consistent principal/share totals, claim status, and numeric limits.

The host supplies a position's aggregate and newest lot. It must establish that
those values belong together, select the actual LIFO head, and retain the
returned lot and position state. The core does not dictate a vector, linked
list, account per lot, lot ID, or capacity. It cannot authenticate data or prove
global totals from an isolated snapshot.

## Data and numbers

Principal and rewards are [`Amount`](src/amount.rs), an alias for `u64`,
throughout arguments, snapshots, balances, and receipts. Each epoch claim floors
its own entitlement independently, with no fractional credit carried to another
claim or epoch. Unpaid whole reward units remain in the epoch accounting; the
host chooses their destination once claims complete.

Shares, entry weights, and penalty fractions use `NonNeg<N>` with
`N: mayf_fixed_number::FixedNumber`. The engine widens integer amounts
internally only when a calculation needs them. It selects the domain's rounding
direction; the numeric driver implements multiplication, division, powers, and
quantum-grid rounding. Withdrawal and claim additionally require the numeric
crate's `IntegralRounding` trait to floor their result to `u64`.

The consumer chooses the driver's precision and range, and the principal and
reward base units. Entry shares round down; proportional share removal and exit
penalties round up; returned principal and settled rewards round down. Integer
complements conserve every principal split. For a successfully settled
positive-share epoch, each user's loss from flooring their share entitlement is
less than one reward unit; no per-user dust is stored.

Funding and snapshot validation reject reward pools that cannot be widened by
the selected driver. Weighted shares must also fit that driver. The Anchor host
uses unsigned Q64.64, which represents the full `u64` reward range. Range checks
and rounding are specified under
[Conservation and rounding](../../README.md#conservation-and-rounding) in the
Reward Clearing overview.

`RewardClearingError` and its [error table](src/error.rs) describe domain
failures. Hosts map these errors and define their own authorization, storage,
custody, and administration failures.

## Test bench

Run from the repository root:

```bash
cargo test -p mayf-reward-clearing --all-targets
```

- [Receipt tests](tests/integration/receipts.rs) demonstrate accounting transitions with
  plain host balances.
- [Differential tests](tests/integration/oracle_differential.rs) compare with a rational
  oracle.
- [Property tests](tests/integration/fuzz_properties.rs) exercise deterministic histories.
- [Accounting bounds](tests/integration/accounting_bounds.rs) cover numeric limits and
  accepted accounting snapshots.

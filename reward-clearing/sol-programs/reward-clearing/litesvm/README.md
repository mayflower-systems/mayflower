# Reward clearing runtime tests

From the repository root:

```sh
cargo test --locked -p mayf-reward-clearing-sol-litesvm
```

The suite reads `../bin/mayf_reward_clearing_sol.so` at runtime, relative to this
crate, and uses `mayf-reward-clearing-sol-cpi` for every instruction and program
account codec. It does not link the program as a native processor. A missing or
invalid binary fails the tests. Run `../build.sh` after changing program source
to refresh the binary and generated clients before testing. Workspace tests and
CI include this crate automatically.

Account-constraint fixes and remaining policy decisions are inventoried in
[`../TODO.md`](../TODO.md). Active regressions cover rejection of native clearing
credits and escrow aliases for withdrawals and claims. They assert exact program
errors and complete rollback, including zero-penalty withdrawals and zero-reward
claims. Native reward tokens remain permitted; the mint restriction applies only
to clearing credits.

LiteSVM 0.18 requires `solana-address ~2.7.0`. The workspace lockfile therefore
selects 2.7.0, while the shared client manifests accept compatible `2.7.0` and
newer 2.x versions. Independent consumers can select 2.9 without changing the
generated account or instruction APIs. One workspace cannot simultaneously
require 2.9 and LiteSVM's 2.7.x range; separate lockfiles would be needed for
independent SDK selections.

## Keep the runtime boundary small

Before adding a case, ask whether an ordinary Rust test could establish the same
fact. Curves, rounding, timing permutations, amount limits and logical model
relationships belong in `mayf-reward-clearing`. Pure host conversions belong in
`prog` unit tests. Use LiteSVM for authenticated accounts, permissions, persisted
updates, allocation, rent, token custody, closure and transaction atomicity.

The suite exercises all thirteen instructions through four modules:

| Module | Responsibility |
| --- | --- |
| `authorization.rs` | Signers, ownership links, wire corruption, account layout, PDA and vault substitution, SPL authority and mint checks |
| `account_constraints.rs` | Native clearing-mint rejection, permitted native rewards, and atomic rejection of self-directed withdrawals and claims |
| `lifecycle.rs` | Creation, counters, administration, permissionless epoch creation and packed-transaction rollback |
| `settlement.rs` | Token movements and burns, stored balances, rent-funded growth, retained capacity, finalization and claim closure |

Simple exact amounts make the expected custody changes readable. The tests do
not compute expectations by calling the domain action being exercised. After
finalization, the stored clearing-credit total is historical; its physical token
escrow has been burned.

## Shared test tools

- `support/addresses.rs` owns state-dependent PDA derivation.
- `support/fixture.rs` owns generated instruction packing. `Fixture::empty()`
  supplies external wallets, mints and balances; `Fixture::new()` creates every
  program account through actual transactions.
- `support/runtime.rs` owns time, signing, typed account reads, corruption and
  snapshots. Signature verification stays enabled. Each transaction uses a fresh
  blockhash so retries reach the program.
- `support/tokens.rs` seeds and reads SPL state with its public codec. The real
  SPL Token program handles transfers and burns.

The fee payer and rent payer are separate wallets. `Bank::reject` asserts the
exact error and instruction index, then checks every supplied account, including
absent addresses, byte for byte. That includes mint supply, token balances,
allocation and rent. Only the fee payer changes, by the reported transaction fee.
`Snapshot` also supports checking unrelated accounts across successful updates.

Add shared helpers when a second test needs them; prefer explicit instruction
arguments and expected state to a configurable scenario engine.

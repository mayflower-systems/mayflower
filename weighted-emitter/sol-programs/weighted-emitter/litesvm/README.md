# Weighted-emitter runtime tests

Run `../build.sh` first, then `cargo test -p mayf-weighted-emitter-sol-litesvm --locked`.
The suite loads the built SBF program and uses its generated CPI client for
instructions and account decoding. Real instructions create every emitter,
gauge, vote position and escrow; only external mints/wallet balances are seeded.
The small `test-program` fixture also executes as SBF to verify that a composing
program can authorize creation and settlement through its own signing PDA.

The shared bank packs and signs transactions with signature verification enabled.
Its rejection helper checks the exact error and instruction index, then checks
all supplied accounts byte-for-byte, including new account allocation, token
balances and mint supply. The fee payer is separate from the rent payer.

These tests cover authority, account links, custody, pending emission, minting,
donations and rollback. `MintingSnapshot` checks changes to token balances and
mint supply against persisted mint counters. It does not calculate expected
emission from rates, elapsed time, weights, or indexes. The pending-emission
test uses the allocation already stored by a vote change to verify that a later
mint consumes it exactly once. The decay-update test checks authorization and
stored configuration without exercising the decay formula.

Before adding a runtime test, identify the Solana effect it needs to observe.
If the assertion concerns a calculation, use a focused ordinary Rust test with
explicit input state and one function under test. Index updates and reweighting
are covered by the pure library's `emit_to_gauge` and `set_gauge_votes` unit tests;
constant-rate and long-interval decay calculations are covered by
`Schedule::advance` unit tests. The library also owns allocation-ratio, numeric
bound, and truncation scenarios.

The overflow-withdrawal regression stays here because it demonstrates the
custody consequence: the program rejects the withdrawal and leaves principal
in escrow. Its numerical cause is tested separately in the library; the open
recovery-policy issue is recorded in [TODO.md](../TODO.md).

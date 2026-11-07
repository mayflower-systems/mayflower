# Reward clearing program

Run `./build.sh` to build all artifacts. The script resolves paths from its own
location, so it also works when invoked from the repository root or elsewhere.
It requires `cargo-build-sbf` and uses Solana platform tools `v1.58` plus the
repository's pinned Rust toolchain and dependencies.

| Directory | Contents |
| --- | --- |
| `prog/` | Anchor program source |
| `bin/` | Built `mayf_reward_clearing_sol.so` |
| `idl/` | Generated `mayf_reward_clearing_sol.json` |
| `cpi/` | Generated Rust library `mayf-reward-clearing-sol-cpi` |

The script builds the program with its default features, extracts its IDL,
generates the CPI bindings using `mayf-anchor-idl-to-gen`, formats them, and
checks that the crate compiles. Cargo's intermediate files and automatic
deployment keypair remain in `target/`.

Keep `cpi/Cargo.toml` as the crate definition. The generated bindings take the
program address through `MayfRewardClearingApi::new(program_id)` and include
instruction builders, account layouts, and argument models.

Tenant and epoch PDA seeds depend on counters stored in accounts. Generation
uses `--manual-pdas`, so callers supply the PDA addresses explicitly. The small
`cpi/src/pda.rs` interface is maintained by hand and preserved on regeneration;
the other files in `cpi/src/` are generated. The program's account constraints
still validate the supplied addresses.

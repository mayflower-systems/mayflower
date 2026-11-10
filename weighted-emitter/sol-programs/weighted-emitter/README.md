# Weighted emitter program

The Solana host for [`mayf-weighted-emitter`](../../rust/mayf-weighted-emitter/README.md).
The pure library owns the schedule and weighted allocation; the program owns
account authorization, vote-token custody, and emission-token minting.

Run `./build.sh` to build all artifacts. The script resolves paths from its own
location, so it also works when invoked from the repository root or elsewhere.
It requires `cargo-build-sbf` and uses Solana platform tools `v1.58` plus the
repository's pinned Rust toolchain and dependencies. The TypeScript steps use
the pnpm workspace, so run `pnpm install` at the repository root first.

| Directory | Contents |
| --- | --- |
| `prog/` | Anchor program source |
| `bin/` | Built `mayf_weighted_emitter_sol.so` |
| `idl/` | Generated `mayf_weighted_emitter_sol.json` |
| `cpi/` | Generated Rust library `mayf-weighted-emitter-sol-cpi` |
| `litesvm/` | Runtime authorization, custody, minting, and rollback tests |
| `ts/` | Generated TypeScript package `@mayflower-sys/weighted-emitter-sol-client` |

The script builds SBF, extracts the IDL, generates both clients with
`mayf-anchor-idl-to-gen`, formats the clients, and checks their builds. It installs
the binary only after those steps succeed. Intermediate artifacts and the
automatically generated deployment keypair stay in `target/`.
The script also builds the small caller program in `litesvm/test-program/` and
places its binary in `litesvm/bin/`. That permissionless fixture exercises real
CPI signing in tests; it is not a production authority.

Keep `cpi/Cargo.toml`, `ts/package.json`, and `ts/tsconfig.json` as the package
definitions. Regenerate client code rather than editing generated instruction
builders, account codecs, or error definitions. Rust clients choose a deployment
with `MayfWeightedEmitterApi::new(program_id)`; TypeScript instruction builders
take the program address as their first argument. Both clients accept explicit
PDA addresses. Rust generation uses `--manual-pdas`, preserving an existing
`cpi/src/pda.rs`; TypeScript generation never derives addresses.

## Custody and emission

An emitter selects a vote mint and an emission mint. They may be the same mint.
Each user's gauge position has its own vote-token escrow, while the gauge tracks
the sum of recorded deposits. Actual token deposits and withdrawals determine
that gauge's weight; unsolicited escrow donations do not create voting power.
Internal bookkeeping is authoritative for principal and withdrawal limits.
After a deposit or withdrawal finishes its token CPI and bookkeeping updates,
the handler reloads the escrow and checks that it still backs the recorded
principal. Token balances are used only for that final invariant check.

Weight changes first accrue the old voting interval through the pure library.
The gauge retains the resulting cumulative allocation even when no emission is
minted in that transaction. Settlement requires the emitter's configured
authority and mints its pending allocation to the receiving token account
selected by that authority. The receiver may have any owner but must use the
emission mint. A composing program can hold the authority through its own PDA.

The emitter's separate mint-authority PDA signs emission minting. Cumulative
allocation and cumulative minted amounts distinguish pending entitlement from
tokens already issued, preventing vote changes from discarding emission or
settlement from issuing it twice. All library state, escrow transfers, and
minting effects commit atomically with their transaction.

## Initializing an emitter

The program supports the original SPL Token program. Wrapped SOL may be used
for voting, but cannot be the emission mint because its supply cannot be minted.
Emission rates are Q64.64 values in raw token units per second; annual decay
uses Q64.64 coefficients as defined by the pure library. Indexes use Q192.64.
Materialized emission and recorded deposits are `u64` values. Settlement floors
fractional raw token units, as specified by the pure library.

1. Choose the new emitter account address and derive its mint-authority PDA.
2. Create the mints and any initial supply. If both roles use the same mint,
   create the initial voting supply before transferring mint authority.
3. Assign the emission mint's mint authority to the emitter PDA using SPL Token.
4. Call `create_emitter`, signed by the payer, new emitter, and configured
   authority. The mint handover and creation can share one transaction.
5. The authority creates gauges; users create a position for each gauge they
   want to vote on, then deposit or withdraw voting tokens.

| PDA | Seeds, in order |
| --- | --- |
| Mint authority | `"mint_authority"`, emitter address |
| Vote position | `"vote_position"`, gauge address, user authority address |
| Vote escrow | `"vote_escrow"`, vote-position address |

All PDAs are derived under the emitter program's address. Emitter and gauge
accounts are supplied as new signer accounts, so a composing program may also
create them with its own PDA signer. The configured authority signs gauge
creation, decay updates, and emission settlement. Users sign their own position
creation, deposits, and withdrawals.

`settle_emission` accepts any receiving token account for the emission mint;
the composing authority is responsible for its routing policy. When the mints
are identical, minting to a vote escrow is also allowed: those tokens are surplus,
not recorded voting principal. Only deposited principal can be withdrawn through
the position; unsolicited surplus is not recoverable through this interface.

An unsafe emission schedule can overflow cumulative `u64` allocation and block
vote withdrawals. The reproduced case and the required recovery-policy decision
are tracked in [TODO.md](TODO.md).

Run `cargo test -p mayf-weighted-emitter --locked` for ordinary Rust accounting
tests. Run `cargo test -p mayf-weighted-emitter-sol-litesvm --locked` after
`./build.sh` for the Solana boundary tests against the built binary.

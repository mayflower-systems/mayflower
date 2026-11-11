# Reward clearing program

Account-constraint fixes and open settlement policy decisions are tracked in
[TODO.md](TODO.md).

Run `./build.sh` to build all artifacts. The script resolves paths from its own
location, so it also works when invoked from the repository root or elsewhere.
It requires `cargo-build-sbf` and uses Solana platform tools `v1.58` plus the
repository's pinned Rust toolchain and dependencies. The TypeScript steps use
the pnpm workspace, so run `pnpm install` at the repository root first.

| Directory  | Contents                                                                 |
| ---------- | ------------------------------------------------------------------------ |
| `prog/`    | Anchor program source                                                    |
| `bin/`     | Built `mayf_reward_clearing_sol.so`                                      |
| `idl/`     | Generated `mayf_reward_clearing_sol.json`                                |
| `cpi/`     | Generated Rust library `mayf-reward-clearing-sol-cpi`                    |
| `litesvm/` | [Runtime boundary tests and shared harness](litesvm/README.md)           |
| `ts/`      | Generated TypeScript package `@mayflower-sys/reward-clearing-sol-client` |

The script builds the program with its default features, extracts its IDL,
generates the CPI bindings using `mayf-anchor-idl-to-gen`, formats them, and
checks that the crate compiles. It then generates the TypeScript client with the
same tool, formats it with Prettier, and compiles it with `tsc`. Cargo's
intermediate files and automatic deployment keypair remain in `target/`.

Keep `cpi/Cargo.toml` as the crate definition. The generated bindings take the
program address through `MayfRewardClearingApi::new(program_id)` and include
instruction builders, account layouts, and argument models.

Keep `ts/package.json` and `ts/tsconfig.json` as the package definition. The
TypeScript config extends the repository's `tsconfig.base.json` unchanged, and
the package depends only on `borsh` and `bs58`. The generated
`create…Instruction` builders take the program address as their first
argument.

Channel group and epoch PDA seeds depend on counters stored in accounts, so
callers supply the PDA addresses explicitly. Rust generation uses
`--manual-pdas`, and the small `cpi/src/pda.rs` interface is maintained by hand
and preserved on regeneration. The TypeScript client never derives PDAs; its
callers derive them with a Solana library, such as `@solana/web3.js`. The
other files in `cpi/src/` and every file in `ts/src/` are generated. Creation
instructions validate derived addresses; subsequent operations use stored
relationships.

## State-machine integration

`create_channel_group` delegates to the library action, which enforces the
one-hour minimum (`MIN_EPOCH_DURATION_SECONDS = 3_600`) and checks that the initial
epoch's end fits in `u64` seconds. The start may be in the past, present, or
future. An Anchor account constraint rejects the native mint (wrapped SOL) for
clearing credits with `NativeClearingMint`, because credit settlement requires
burns that native token accounts do not support. Native reward tokens remain
permitted: reward settlement uses transfers. Anchor creates the returned group
and initial epoch with their custody accounts and preserves the organization,
mint, and PDA metadata.

All live actions use the library's `Epoch::validate_active(group, now)`: the
epoch must belong to the supplied group, match its current counter, and contain
the authenticated time in `[start, end)`. This applies to channel and channel
epoch creation, bidder position creation, deposits, withdrawals, and reward
funding. The counter cannot replace the clock check: an ended epoch can remain
current until someone advances the group. The clock cannot replace the group
and counter checks: an epoch must also be the group's recorded current epoch.
Historical finalization and claims deliberately do not require a current epoch.

`channel_create` calls the library's `create_channel` action with the channel
group and active epoch. The library validates the epoch's group, current
counter, and live interval, then returns a new channel and its first channel
epoch. Anchor creates both accounts and their escrows atomically. Every channel
therefore starts live with a mandatory `last_created_epoch_counter: u32`.

Later `channel_epoch_create` calls use `create_channel_epoch`. The library
requires the group's current live epoch to be strictly later than the channel's
last created epoch, and rejects disabled channels. A channel can skip epochs in
which it does not participate. The handler persists the returned channel epoch
and updated channel counter. Anchor supplies authenticated time and handles
account creation and token custody.

`epoch_create` converts the channel group and current epoch, calls
`advance_epoch`, and persists the returned epoch and group index. The library
owns timing checks, counter progression, and the next epoch's schedule, including
checking that its end does not overflow. Anchor keeps the group's organization,
mint, and PDA metadata, and marks the group writable for the update. Stored
durations remain `u32` seconds; conversion from
the library's `u64` durations is checked.

`channel_epoch_finalize` calls the library's `finalize_channel_epoch`, burns the
tracked clearing credits, and stores the returned state in the existing channel
epoch. Finalization is permissionless and creates no account: it needs neither
a payer nor the system program, and takes no channel-group account. The library
validates the epoch relationship, deadline, and one-time transition, then fixes
the original reward pot and total weighted shares in `final_totals`.
`final_totals.is_some()` records completion, including an empty epoch;
the deadline alone only determines when finalization is allowed. Funding rejects
already-finalized epochs. Channel creation and default-term updates validate
their inputs through the library before storing them.

`channel_epoch_add_to_reward_pot` calls `add_reward` before transferring tokens.
The action validates the channel, group and epoch relationships, live interval,
unfinalized state, and balance arithmetic. The handler transfers the requested
amount and persists the returned state. It takes the channel account and needs
no separate payer. Deposits and withdrawals also reject finalized epochs in
their library actions.

`bidder_position_create` calls `create_bidder_position` to create an empty,
authority-owned position. The library requires a matching, unfinalized channel
epoch in the group's current live epoch. Its lots are stored together as a Borsh
`Vec`, oldest first.
`bidder_position_deposit` passes `MAX_LOT_COUNT: u32 = 8` to the library action.
Anchor checks the signer against the stored authority. The library checks the
position's channel and epoch counter against the supplied channel epoch, and
checks that channel epoch against the supplied channel and epoch. Deposits do
not rederive the position's address from seeds.
After the action succeeds, the handler funds any rent shortfall and grows the
position account only to fit the returned lots, then transfers credits and
persists both the position and channel epoch. A failed transfer rolls back the
allocation and rent payment along with the transaction.

`bidder_position_withdraw` takes the gross credits to remove from the position
and calls `withdraw_credits`. The library consumes lots newest first, calculates
the exit penalty, and returns both updated states and the exact credits to return
and forfeit. Anchor burns the forfeited credits from the channel epoch's escrow
and transfers the remainder to the destination chosen by the bidder's authority.
The destination can have any owner, but an account constraint rejects the source
escrow as destination with `WithdrawalDestinationIsEscrow`, even when no penalty
would be burned. SPL Token validates mint compatibility and
authorization for transfers and burns. Both token operations are signed by the
channel epoch PDA. The handler then persists both states; a failed token operation
rolls back the entire withdrawal, including any burn. Authority and custody checks
stay in Anchor; channel and epoch checks stay in the library.

`bidder_position_claim` calls `claim` with the position and its finalized channel
epoch. The library pays `floor(final_reward_pot * position_shares / final_shares)`
using fused multiply/divide, without first rounding a per-share rate. It marks
the position claimed and subtracts its shares and payout from the remaining
balances. The frozen totals never change, so claim order does not affect payouts.
Anchor checks the position authority, transfers the reward to the chosen token
account, persists the returned channel epoch, and closes the position account,
returning its lamports to the authority. The destination can have any owner.
A constraint rejects the reward escrow as destination with
`ClaimDestinationIsEscrow` before payout or closure, including for a zero payout.
A zero payout still retires the shares and closes the position; it skips the
token transfer. A failed transfer rolls back the state update and closure.
Claiming requires surviving shares; an empty position cannot close this way.
The library marks the consumed position claimed. The host cannot recreate it
for that finalized or expired epoch because creation validates the lifecycle.

Claims have no deadline and do not require the channel group to have advanced.
They rely on the stored finalization snapshot and do not burn credits again.
A client can place `channel_epoch_finalize` followed by `bidder_position_claim`
in one transaction. Finalization handles one channel epoch; it does not sweep
other channels automatically. Unclaimed rewards and rounding remainder stay in
the reward escrow; these instructions do not close the channel epoch or sweep
its remainder.

An empty position occupies 82 bytes, including its discriminator and Vec length.
Each lot adds 24 bytes: `u64` credits and raw Q64.64 shares in a `u128`. Thus one
lot occupies 106 bytes and eight lots occupy 274 bytes. The account is not
preallocated for eight lots. The containing account supplies each lot's position
identity when converting to the library model.
Withdrawals remove exhausted lots from the Vec but retain the allocated space
for reuse by later deposits.

Business rules and domain relationships belong to the library actions. Anchor
constraints cover account ownership, authorization, PDA creation, and token
custody, including supported clearing mints and distinct payout destinations.
Existing token accounts used only for CPI are passed as `UncheckedAccount`;
SPL Token validates them. `token::mint` and `token::authority` configure newly
created escrows. Domain relationships already validated by library actions are not
checked again in Anchor constraints. The exhaustive
`From<RewardClearingError> for RewardClearingProgramError` mapping in
`prog/src/error.rs` translates domain failures into Anchor error codes; handlers
apply it explicitly at library calls.

Handlers that need authenticated time obtain it through `clock::now_unix_seconds()`;
clock errors and negative timestamps propagate as errors.

The adapters use account addresses as library identities and `U64F64` for
numeric state. Channel defaults remain in micro-basis points; conversion
validates the terms and rounds the slope down once. Each channel epoch stores
that exact Q64.64 snapshot. Its optional `StoredFinalizedTotals` holds the original
whole reward pot and Q64.64 weighted shares, distinguishing an unfinalized epoch
from a finalized epoch even when either total is zero. Account-to-model conversions
use `TryFrom` where stored terms need validation.

Channel epoch and escrow PDAs use their respective seed prefix, the channel
address, and the epoch counter encoded as four little-endian bytes. The channel
account is writable during epoch creation to record `last_created_epoch_counter`.
The clearing-credit escrow prefix is `channel_epoch_credits_escrow`, within
Solana's per-seed length limit. Bidder position PDAs use `bidder_position`, the
channel address, the four-byte epoch counter, and the bidder authority.

Run `cargo test -p mayf-reward-clearing` for the business-rule tests; these do
not require a Solana runtime.

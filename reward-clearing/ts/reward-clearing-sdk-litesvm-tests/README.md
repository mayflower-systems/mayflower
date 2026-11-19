# Reward Clearing SDK runtime tests

Tests the public `@mayflower-sys/reward-clearing-sdk-ts` API against the compiled Solana program in LiteSVM. The examples use `@solana/web3.js` v1 to build, sign, and submit real transactions. They cover account reads, persisted writes, token transfers and burns, rent, closure, authorization, and rollback. Auction mathematics stays in the Rust unit tests.

From the repository root, using the workspace's Node 26 toolchain:

```sh
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @mayflower-sys/reward-clearing-sdk-litesvm-tests... build
pnpm --filter @mayflower-sys/reward-clearing-sdk-litesvm-tests test
```

The suite loads `../../sol-programs/reward-clearing/bin/mayf_reward_clearing_sol.so` relative to this package, independently of the working directory. This is the artifact produced by the program's `build.sh`. Missing or invalid binaries fail the suite. After changing the program, run `reward-clearing/sol-programs/reward-clearing/build.sh` to refresh the binary and generated client before testing.

LiteSVM is pinned to 0.2.0 for its web3.js v1 interface and compatibility with x86-64 Ubuntu 22.04 (glibc 2.35). The 0.6.0 GNU/Linux binding references `__isoc23_strtol`, which this host does not provide; its loader misreports that failure as a missing native binding. Keep the pinned version and lockfile intact rather than deleting dependencies or enabling install scripts. The test command needs no alternate libc, native-library override, or runtime download.

LiteSVM 0.2 retains closed accounts as zero-lamport, empty system accounts until cleanup. The claim test checks those closure conditions, the full rent refund, and rejection of a second claim; it also accepts a purged account returned as `null`.

## Client examples

[`test/sdk.test.ts`](test/sdk.test.ts) exercises all 13 SDK builders, including the factory bound to the program address. Each case is self-contained and shows explicit instruction arguments and signers:

- Creation reads organization and group counters before deriving the next addresses, then reads all six program account models and checks their token vaults.
- Administration updates defaults and opens the next epoch with an unrelated payer, preserving the existing epoch's terms.
- Deposits and withdrawals check token custody, mint supply, bidder lots, sponsored account growth, and retained allocation.
- Settlement advances the group, finalizes an older epoch, and claims its rewards and position rent.
- Authorization and rollback cases check exact errors and unchanged accounts after rejected transactions. Hostile messages deliberately change account metadata to exercise on-chain checks.

[`test/prelude/accounts.ts`](test/prelude/accounts.ts) shows the read boundary: account bytes go through a public generated decoder, then the SDK's `fromAccount` conversion. The remaining prelude modules create users, initialize and mint real SPL tokens, create program accounts through SDK instructions, and sign transactions. Mint helpers return the mint authority keypair for later funding.

The Node test runner uses one process and one LiteSVM instance, with concurrency limited to one. Every test creates fresh users, mints, token accounts, and an organization, and explicitly sets the clock. No program state is injected. Turbo test caching is disabled so the suite always executes the current binary; the TypeScript CI job runs it after building the workspace.

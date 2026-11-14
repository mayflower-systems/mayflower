# Reward clearing on Solana

The Anchor program runs reward-clearing auctions with SPL Token escrow.
It applies the Rust state machine using Solana's clock and settles token
movements and account updates in the same transaction.
[Reward-clearing design](../../rust/mayf-reward-clearing/README.md)

## Calling the program

Create an organization, a channel group with a clearing mint and epoch
schedule, and a reward channel. Channel creation opens the first auction in
the group's live epoch. The organization authority creates groups and
channels and can set channel defaults for future auctions. For subsequent
epochs, any payer can advance the group and open an enabled channel for its
live epoch. [Instruction entry points](prog/src/lib.rs)

Fund the pot with `channel_epoch_add_to_reward_pot` so the transfer is counted
in the auction. A direct transfer to the vault leaves the recorded pot
unchanged. The reward mint may be wrapped SOL. The clearing mint must support
burns, so the program rejects wrapped SOL as clearing credits.
[Funding handler](prog/src/instructions/channel_epoch_add_to_reward_pot.rs) ·
[Group creation handler](prog/src/instructions/create_channel_group.rs)

Bidders sign for their own positions, which can hold up to eight live deposit
lots. Deposits grow the account as needed at the payer's expense. Withdrawals
free lot capacity and retain the allocated space for reuse.
[Deposit handler](prog/src/instructions/bidder_position_deposit.rs) ·
[Position layout](prog/src/state/bidder_position.rs)

Once the deadline passes, anyone can finalize the channel epoch and burn its
remaining committed credits. A bidder can finalize and claim in one
transaction. Claiming requires the position authority and closes the bidder's
account, returning its lamports to the authority even when its payout rounds
to zero. Unclaimed rewards and any remainder stay in the pot.
[Finalization handler](prog/src/instructions/channel_epoch_finalize.rs) ·
[Claim handler](prog/src/instructions/bidder_position_claim.rs)

The generated Rust and TypeScript builders use an explicit program address
and take account addresses from the caller. Channel group and epoch addresses
depend on counters stored on chain. The runtime test fixture shows how to
derive them and build transactions.
[Address derivation](litesvm/tests/support/addresses.rs) ·
[Transaction fixture](litesvm/tests/support/fixture.rs)

## Build and verify

From the workspace root, install the JavaScript dependencies and run the build
with `cargo-build-sbf` available. The script builds the program with Solana
platform tools `v1.58` and regenerates the IDL and both clients.

```bash
pnpm install
./reward-clearing/sol-programs/reward-clearing/build.sh
cargo test --locked -p mayf-reward-clearing-sol-litesvm
```

The LiteSVM suite exercises the built program through generated instructions,
including custody, authorization, rent, and rollback.
[Runtime tests](litesvm/README.md)

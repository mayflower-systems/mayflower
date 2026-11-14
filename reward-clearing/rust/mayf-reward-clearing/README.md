# mayf-reward-clearing

`mayf-reward-clearing` is a Rust state machine for distributing a funded reward
pot through a timed auction. Bidders put clearing credits in escrow for
weighted shares of the pot. At finalization, the credits still committed are
burned. Each position can then claim its fraction of the reward in proportion
to its surviving shares. The reward can be a different asset from the credits.

## Why put credits at stake?

On a public blockchain, accepted bids and funding are visible as they arrive.
A bidder can wait until the last moment to see how much is in the pot and how
many shares are competing for it. That bidder can then decide whether and how
much to commit while giving others little time to respond. This is sniping.

With free cancellation, a large temporary bid can make the reward per share
look unattractive. A bidder can use this kind of **stink bid** to discourage
competition, then pull the bid after others have decided to stay away.

In reward clearing, every bid puts credits at risk. A withdrawal gives up the
corresponding shares and returns the withdrawn credits after a penalty that
is burned. The penalty starts at zero and rises toward the deadline.
Keeping a bid through the deadline commits the full principal to be burned at
finalization.

## Why two timing rules?

Let `u = elapsed / duration` be the elapsed fraction of the scheduled epoch.
An auction keeps the terms set when it opens and uses the same epoch clock for
every bid.

- **Entry bonus.** A deposit gets `1 + b * (1 - u)` shares per credit in ideal
  arithmetic, with `b` between 0 and 2. The bonus rewards committing before
  later funding and bids are known. A linear decline gives each second of
  waiting the same cost in potential shares. With `b = 1`, a credit buys two
  shares at the start and one and a half halfway through. A deposit keeps
  the shares it earned as the bonus declines for later deposits.
- **Exit penalty.** A withdrawal forfeits a fraction `u^k` of the requested
  credits in ideal arithmetic, with `k` from 1 to 4. With `k = 2`, the penalty
  is 1% at 10% elapsed, 25% halfway through, and 81% at 90% elapsed. The
  quadratic curve allows cheaper revisions early, when other bidders have
  time to respond. Late in the auction, a bidder has seen more of the market's
  response to their commitment, and others have little time to reconsider
  after a withdrawal. The steeper rise prices that late reversal. Even a
  deposit made moments earlier pays the current penalty rate on withdrawal.

A large late bid can still earn more shares than a small early bid. Payouts
depend on actual funding and competition.
[Revenue Clearing Rationale](../../../docs/github-wiki/content/Revenue-Clearing/Revenue-Clearing-Rationale.md)

## Using the library

Create a channel group for the schedule, then create a channel during the
group's live epoch to open the first auction. Open a position for each
bidder. Funding, deposits, and withdrawals happen during the live window.
Finalize at or after the deadline, then claim for positions with surviving
shares.

Use the public actions for these transitions. Each takes the relevant state
and returns a receipt with the next state and any computed settlement amounts.
The host loads the current records, supplies trusted execution time, and
commits the receipt together with the required token movements.
[Actions](src/actions.rs) · [Crate documentation](src/lib.rs)

From the workspace root, run the auction example, open the API docs, or run
the state-machine tests.

```bash
cargo run -p mayf-reward-clearing --example auction
cargo doc -p mayf-reward-clearing --open
cargo test -p mayf-reward-clearing --all-targets
```

The example follows two bidders through deposit, a partial withdrawal,
finalization, and claims. The wiki specifies the precise rounding and
settlement rules.
[Auction example](examples/auction.rs) ·
[Formal specification](../../../docs/github-wiki/content/Revenue-Clearing/Revenue-Clearing-Formal-Specification.md)

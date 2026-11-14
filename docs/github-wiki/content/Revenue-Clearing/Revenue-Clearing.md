<!--
title: Revenue Clearing
summary:
  How a public auction turns committed clearing tokens into weighted shares of
  a funded reward pot.
tags:
  - revenue-clearing
  - tokenomics
-->

Revenue clearing distributes a reward pot among bidders willing to spend a
**clearing token** for a portion of it. The pot can hold a different asset
from the clearing token. Bidders decide how much that reward is worth to them.
Settlement uses the funded amount and the shares bidders have earned.

## Why require a commitment?

Everyone can see the accepted bids and funding on a public chain. A bidder
who waits until the last moment gets to see more of the auction while giving
competitors little time to respond. With free cancellation, a large temporary
bid can make the reward per share look too small to compete for, even though
the bidder is unwilling to pay for those shares.

Revenue clearing puts tokens at stake in both decisions. Bidders who enter
early can earn more shares per token while the final pot and competition are
still uncertain. A bidder can revise a commitment by withdrawing, but doing
so forfeits part of the withdrawn tokens. As the deadline approaches, that
forfeiture grows. This makes a visible bid costly to abandon after others
have acted on it. [Revenue Clearing Rationale](Revenue-Clearing-Rationale)

## From funding to claim

A **channel** runs auctions for a particular reward asset. Each auction uses
a fixed window called an **epoch**. Auctions in the same channel group share a
schedule, and each channel epoch has its own funded pot and positions.

1. During the live window, funders add rewards and bidders deposit clearing
   tokens as **principal**. Each deposit creates a **lot** whose shares encode
   the entry bonus at the time of deposit.
2. A withdrawal removes principal and its associated shares from the newest
   lots first. The bidder receives the refund after the exit penalty, and
   the forfeited clearing tokens are burned.
3. Deposits, withdrawals, and funding close at the deadline. Anyone can then
   finalize the auction. Finalization burns the remaining principal and
   freezes the funded reward and surviving shares.
4. A position with surviving shares can claim once, with no claim deadline.
   Its payout is the same fraction of the final pot as its fraction of the
   surviving shares, rounded down to a whole reward unit.

Accepted funding can arrive throughout the auction. At any given pot size,
a deposit dilutes the share of existing positions and a withdrawal increases
it. Withdrawal forfeiture takes clearing tokens out of circulation and adds
nothing to the reward pot. If the pot is empty at finalization, the committed
tokens still burn. If no shares survive, the whole reward pot remains in
escrow.

- [Time Weight](Revenue-Clearing-Time-Weight) shows the linear entry bonus,
  quadratic exit example, and two bidders' balances through settlement.
- [Formal Specification](Revenue-Clearing-Formal-Specification) gives the
  exact lifecycle, equations, and rounding rules.

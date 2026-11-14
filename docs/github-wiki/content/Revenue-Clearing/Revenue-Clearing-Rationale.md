<!--
title: Revenue Clearing Rationale
summary:
  How public bidding creates incentives to snipe or mislead other bidders,
  and why entry rewards and exit penalties use different curves.
tags:
  - revenue-clearing
  - tokenomics
-->

On a public blockchain, accepted commitments and funding reveal the current
pot and outstanding shares. With a fixed deadline, a bidder who waits gets
to see more of that activity and gives others less time to react. Future
bids, funding, and transaction inclusion remain uncertain.

**Sniping** exploits the fixed close. If a token buys the same number of shares
throughout the auction, a bidder can watch the pot and competing shares grow
and decide at the last moment whether to participate. An early bidder commits
while those values are still changing and exposes their bid for others to react
to.

A **stink bid** is a large apparent commitment that makes the reward per share
look unattractive to other bidders. If it is free to cancel near the deadline,
the bidder can recover the principal after others have decided to sit out or
commit less. The bid influenced their choices despite carrying little cost
for the bidder who posted it.

## Make the bid cost something

In revenue clearing, a deposit puts actual clearing tokens in escrow. The
deposit increases visible competition and creates a claim on the pot if its
shares survive. A withdrawal gives up the shares backed by the withdrawn
principal and burns a fraction of that principal. Any clearing tokens still
committed at finalization are burned in full.

Burning the principal keeps the payment for shares separate from the funded
reward asset. Bidders supply their own valuations when bidding, and the pot
pays out in the asset it received. [Revenue Clearing](Revenue-Clearing)

## Why the entry bonus is linear

Let $`u=t/T`$ be the fraction of the epoch that has elapsed. A new deposit earns
the following ideal number of shares per clearing token.

```math
w(u)=1+b(1-u),\qquad 0\le b\le2.
```

Entering early earns extra shares in exchange for committing before the final
pot and competition are known. Waiting lets a bidder observe more data or
avoid committing entirely if the auction turns out to be unattractive.
The linear curve gives up the same amount of potential weight for each second
of waiting. This makes the tradeoff predictable even when bids and funding
arrive in bursts. With $`b=1`$, one token buys two shares at the start and
1.5 halfway through. A deposit retains its own shares as the entry bonus falls
for later deposits.

The slope $`b`$ sets the size of that incentive. At zero, every deposit has
unit weight. At the maximum slope of two, a deposit at the start receives
three shares per token. The baseline of one near the deadline leaves late
bids able to compete.

## Why the exit penalty can be quadratic

Early in the epoch, other bidders have time to react when new funding or
competition prompts someone to revise a bid. Later, a bidder has had longer
to observe how others responded to their displayed commitment. Canceling
close to the deadline can leave others little time to revise decisions made
under that apparent demand.

The ideal penalty on the withdrawn principal is

```math
p(u)=u^k,\qquad k\in\{1,2,3,4\}.
```

With $`k=2`$, the ideal penalty rises by 6.25 percentage points over the first
quarter of the epoch and by 43.75 points over the last quarter, up to the
deadline limit. This gives bidders room to adjust early as the pot and other
bids take shape. It makes the option to keep a bid visible until others have
acted on it expensive to exercise. The curve uses elapsed epoch time as a
proxy for that opportunity. It applies equally to a fresh lot and an old lot
withdrawn at the same time.

An exponent of one gives a constant rate of increase. Larger exponents lower
the penalty between the endpoints and concentrate more of the rise near the
deadline. A channel can reward early entry strongly and still give bidders
room to revise. At $`b=2`$ and $`k=4`$, a deposit at the start earns three
shares per token and a halfway withdrawal has an ideal penalty of 6.25%.
[Time Weight](Revenue-Clearing-Time-Weight)

## What still depends on the auction

- Late bids can still earn more shares by committing more tokens. Inclusion
  and ordering still determine which actions land before the deadline.
- A sole bidder with positive shares receives the whole funded pot. Economic
  competition depends on other bidders showing up and committing tokens.
- The funded reward and surviving shares determine the payout. Promised
  funding carries no claimable balance, and remaining principal burns even
  when the funded pot is empty.

Every claim uses the same final pot and share total. Bidders can calculate the
effect of an accepted action from public state, while the eventual payout
depends on the rest of the auction.
[Formal Specification](Revenue-Clearing-Formal-Specification)

<!--
title: Revenue Clearing Time Weight
summary:
  How the clearing auction's entry weight, withdrawal penalty, and
  last-in-first-out deposit lots affect committed tokens and reward shares.
tags:
  - revenue-clearing
  - tokenomics
-->

In a [clearing auction](Revenue-Clearing), a participant commits clearing tokens
for shares of an epoch's reward pot. Entry time determines the weight of a
deposit; withdrawal time determines the penalty on the requested principal. Each
deposit has its own lot. A withdrawal removes principal and shares from the
newest lots first while preserving the shares of untouched older lots.

## Entry weight

Let $`T`$ be the epoch duration and $`t`$ the time since its start. For a
deposit before the deadline, the ideal entry weight is

```math
w(t)=1+b\left(1-\frac{t}{T}\right),\qquad 0\le b\le2.
```

A deposit of $`a`$ clearing-token units earns $`a\,w(t)`$ shares in ideal
arithmetic. The entry weight applies once, at deposit. The resulting shares stay
with that lot until withdrawn or claimed.

At $`b=0`$, weight is always 1. Increasing $`b`$ increases the early-entry
advantage, up to a starting weight of 3 at $`b=2`$. Fixed-point rounding can
make nearby entry times tie. [Formal Specification](Revenue-Clearing-Formal-Specification)

## Withdrawal penalty

For a withdrawal at elapsed time $`t`$, the ideal penalty fraction is

```math
p(t)=\left(\frac{t}{T}\right)^k,\qquad k\in\{1,2,3,4\}.
```

Withdrawing $`x`$ units returns $`x(1-p(t))`$ in ideal arithmetic. The
withdrawal burns the forfeited $`xp(t)`$ and destroys the shares backed by
the withdrawn principal. Both curves use time since the **epoch start**.
Two lots withdrawn at the same time pay the same penalty rate.

## Comparing the curves

The table shows ideal values before rounding. With $`b=1`$, the entry weight
falls by 0.25 shares per token every quarter of the epoch. The quadratic
penalty rises by more each quarter. The deadline row shows the curves'
endpoints after bidding has closed.

| Epoch elapsed | Shares per token ($`b=1`$) | Linear penalty ($`k=1`$) | Quadratic penalty ($`k=2`$) |
| ------------: | -----------------------: | ---------------------: | ------------------------: |
|            0% |                     2.00 |                     0% |                        0% |
|           25% |                     1.75 |                    25% |                     6.25% |
|           50% |                     1.50 |                    50% |                       25% |
|           75% |                     1.25 |                    75% |                    56.25% |
|           90% |                     1.10 |                    90% |                       81% |
|          100% |                     1.00 |                   100% |                      100% |

The entry weight answers how many shares a new deposit will buy. The penalty
answers how much of an existing commitment a bidder can recover. The example
below uses $`b=1`$ and $`k=2`$. [Revenue Clearing Rationale](Revenue-Clearing-Rationale)

## Lots and withdrawal order

Every deposit creates a lot with its own principal and shares. A position's lots
are last-in-first-out: one withdrawal can consume the newest surviving lot and
continue through older lots until it has removed the requested principal. Since
entry weight is non-increasing, the newest lot has the lowest weight or ties
with an older lot.

In ideal arithmetic, withdrawing a fraction of a lot's remaining principal
removes the same fraction of its remaining shares. Thus the newest lot removes
the fewest shares for a given amount of principal, preserving older,
higher-weight lots. Actual accounting rounds partial share removal up and the
returned principal down. The refund is calculated once for the entire requested
amount, even when it spans lots. Splitting the request into separate actions can
reduce the refund: at a 25% penalty, one two-unit withdrawal returns one unit,
while two one-unit withdrawals return zero each.

For example, suppose a position holds an older lot of 100 principal and 200
shares, then a newer lot of 100 principal and 150 shares. Withdrawing 150
principal removes all 150 shares of the newer lot and 100 shares of the older
lot. The position keeps 50 principal and 100 shares. At a 25% penalty, the
request returns $`\lfloor150\times0.75\rfloor=112`$ principal and burns 38.

## Worked example: withdrawal through settlement

The quantities below are whole token base units. The times and amounts give
exact entry weights and refund arithmetic.

1. At the start, Alice deposits 100 clearing units at weight 2 and receives 200
   shares.
2. Halfway through, Bob deposits 100 clearing units at weight 1.5 and receives
   150 shares. He then withdraws 40 units from that lot at the same halfway
   point. The 25% penalty forfeits 10 units and returns 30. His withdrawal
   removes 60 shares, leaving 60 units of principal and 90 shares.
3. A funder adds 90 reward units to the pot. Assume no further activity during
   the live window.
4. Finalization burns Alice's 100 and Bob's remaining 60 clearing units. The
   frozen pot is 90 reward units and total surviving shares are $`200+90=290`$.
5. Alice claims $`\lfloor90\times200/290\rfloor=62`$ reward units; Bob claims
   $`\lfloor90\times90/290\rfloor=27`$. After both claims, the remaining reward
   unit stays in escrow.

The 200 clearing units deposited are accounted for as 30 returned to Bob, 10
burned on withdrawal, and 160 burned at finalization. The 90 reward units are
accounted for as 89 in participant payouts and one unit of remainder. Alice's
fraction of the pot increased when Bob removed shares, but his 10-unit
clearing-token penalty did not add reward to the pot. Clearing and reward units
may be different assets, so their amounts are not added together.

The example uses the same frozen pot and share total for both claims.
[Formal Specification](Revenue-Clearing-Formal-Specification#finalization-and-claims)

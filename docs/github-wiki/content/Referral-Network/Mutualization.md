<!--
title: Mutualization
summary:
  A parent's discretionary marketing budget comes from gross referral overrides
  and is divided among immediate children by their escrowed credits.
tags:
  - referral
  - tokenomics
-->

In the target [Referral Network](Referral-Network), mutualization is a parent's
voluntary marketing budget. The parent gives up some of its downline commissions
to make its family more attractive. Immediate children compete for that budget
by escrowing credits.

## What funds the pool

Consider an interval with unchanged marketing fraction and child escrow balances.
Let $O$ be the parent's **gross referral override income**, in credits, during
that interval. It includes commissions from every permitted descendant generation
and every asset lane. The parent chooses a fraction $\mu\in[0,1]$.

If any immediate child has credits escrowed, the marketing pool is $\mu O$.
Children do not receive a fraction of the parent's own-asset income, admission
fees, or incoming family gifts. Those amounts remain outside this pool.

The pool redistributes credits already allocated to the parent. It does not add
emissions, transfer deposited assets, or consume the children's escrow principal.

## How children divide it

If child $c$ has $s_c$ credits escrowed, and the immediate children together have
$S=\sum_c s_c$, then for $S>0$:

$$G_c=\mu O\frac{s_c}{S}.$$

The same credit escrow divides the combined override earnings from all lanes.
A child does not need BTC to receive a gift funded by BTC, or ETH to receive one
funded by ETH. Raw asset balances and the number of child accounts do not enter
this division.

A child with zero credit escrow receives no gift. If $S=0$, the parent keeps all
of its override income for that interval. A child that stakes later receives
nothing from that earlier interval.

For example, a parent earning 120 override credits chooses $\mu=25\%$, creating
a 30-credit pool:

| Child | Credits escrowed | Fraction of pool | Gift |
| --- | ---: | ---: | ---: |
| Bob | 100 | 1/3 | 10 |
| Carol | 200 | 2/3 | 20 |
| Dave | 0 | 0 | 0 |

The parent keeps the other 90 override credits, all of its own-asset earnings,
and any gift from its own parent. Splitting Bob's 100 credits across sibling
accounts holding 40 and 60 gives those accounts 4 and 6: still 10 in total.

## Why a nephew can help an uncle

Suppose Bob and Carol are children of Alice, and Bob recruits another asset
holder. That new descendant can increase Alice's gross overrides. Carol can
receive part of the larger marketing pool through the credits she has escrowed,
even though the new member is outside Carol's own downline.

The new member's assets do not themselves increase Alice's immediate-child
credit-escrow denominator. This is the family exposure: members can benefit from
productive branches beside their own. A new sibling who stakes credits does
increase the denominator and competes for the pool.

The actual income still depends on global participation, lane allocations, and
Alice's chosen marketing fraction. Growth does not guarantee that every
member's total credits increase.

## Gifts stop at their recipient

If Alice gives Bob 10 credits, Bob keeps that gift in full. Bob's own marketing
pool is calculated only from Bob's gross downline overrides. Alice's gift neither
enters Bob's pool nor pays another upline commission.

Receiving a gift also does not automatically increase Bob's escrow. He can
collect the credits and explicitly restake them for subsequent income.

## A changing offer with withdrawable principal

A parent can change $\mu$ at any time for future accrual, including setting it to
zero. A current percentage is a discretionary commercial offer, not ownership of
a permanent fraction of future earnings. The parent may participate through its
own child accounts under the same escrow rules.

Children can withdraw some or all of their escrow without a mandatory cooldown
or forfeiture. Their already earned credits remain theirs. Moving credits to a
different family requires a node under that family; an existing parent
connection does not move.

Every escrow or percentage change settles earlier income first, across **all**
lanes funding that family. This prevents new stake from capturing an old pool.
[Emission Accounting](Emission-Accounting) gives the settlement rule, and
[Link Shopping](Link-Shopping) explains how to compare offers.

<!--
title: Revenue Clearing Formal Specification
summary:
  Mathematical rules for revenue clearing: epoch parameters, funding, weighted
  deposits, withdrawals, settlement, rounding, and conservation.
tags:
  - revenue-clearing
  - tokenomics
  - math
-->

Participants commit clearing tokens for shares of a funded reward pot. Entry
time determines the shares created. Withdrawal time determines the forfeiture
of principal. Finalization fixes the reward pot and surviving shares for
proportional claims. [Revenue Clearing Rationale](Revenue-Clearing-Rationale) ·
[Time Weight](Revenue-Clearing-Time-Weight)

## Epoch and parameters

An epoch begins at $`start`$ and lasts $`T\ge3{,}600`$ seconds. Deposits,
withdrawals, and funding are accepted in the live interval $`[start,start+T)`$.
Write $`t=now-start`$ for elapsed time.

Each auction fixes two parameters.

- **Entry bonus slope** $`b`$, with $`0\le b\le2`$. The ideal entry weight
  starts at $`1+b`$ and declines linearly toward 1. The reference value is
  $`b=1`$.
- **Exit penalty exponent** $`k\in\{1,2,3,4\}`$. It determines the shape of the
  withdrawal penalty. The reference $`k=2`$ gives a quadratic penalty, and
  $`k=1`$ gives a linear penalty.

The duration $`T`$ belongs to the channel group's immutable schedule. A
channel epoch copies the channel's configured $`b`$ and $`k`$ when it opens
and keeps those terms. In the Solana program, the organization authority can
update the channel defaults for future auctions. The caller supplies the terms
explicitly. Disabling a channel prevents new auctions while its existing
auctions remain usable.

For a repeating schedule with master start $`t_0`$ and duration $`T`$, the epoch
containing $`now\ge t_0`$ is

```math
n=\left\lfloor\frac{now-t_0}{T}\right\rfloor,\qquad
start_n=t_0+nT,\qquad end_n=start_n+T.
```

The group's recorded epoch advances one counter at a time after the current
epoch ends. A live action must refer to that recorded current epoch and execute
in its live interval. Creating a channel also opens its first auction in the
group's current live epoch. A channel may skip later epochs and opens at most
one auction per epoch. An auction opened partway through an epoch uses the
group epoch's start for both curves. Each auction has its own pot, positions,
and shares. Earlier auctions can be finalized and claimed independently of
later ones. Timestamp additions and counter increments must fit their storage
ranges.

## Quantities and rounding

During a live epoch, let $`A`$ be the committed principal, $`S`$ the surviving
shares, and $`R`$ the funded net reward. A channel epoch opens with
$`A=S=R=0`$ and no finalized totals. A new position holds an empty, ordered
list of deposit lots. Each lot records its remaining principal and shares.
The shares created at deposit stay with that lot until withdrawn or claimed.

Principal and rewards are accounted in whole base units of their respective
assets. Shares, weights, and fractions use a fixed-point grid of quantum
$`\varepsilon`$. Write $`\lfloor x\rfloor_\varepsilon`$ and
$`\lceil x\rceil_\varepsilon`$ for rounding down and up to that grid, and
$`\lfloor x\rfloor_{unit}`$ and $`\lceil x\rceil_{unit}`$ for rounding to whole
asset units. The current Solana arithmetic uses $`\varepsilon=2^{-64}`$. The
configured slope is an integer $`b_{cfg}`$ in millionths, with
$`0\le b_{cfg}\le2{,}000{,}000`$; the snapshotted numeric slope is
$`b=\lfloor b_{cfg}/10^6\rfloor_\varepsilon`$.

Rounding is directed consistently: entry weight and shares round down; the
withdrawal penalty and partial-lot share removal round up; returned principal
and claims round down to whole units. These choices prevent fractional
accounting from creating additional whole-unit claims. Amounts and timestamps
are unsigned 64-bit integers; epoch counters are unsigned 32-bit integers.
Shares and intermediate results must fit the selected fixed-point range.
A failed range or relationship check rejects the transition. An integration
must commit accounting and token settlement atomically.

## Funding

A funder adds a whole amount $`N\ge0`$ of the reward asset during the live
interval:

```math
R'=R+N.
```

Exactly $`N`$ enters the reward escrow. Clearing charges no funding fee and
accepts no maximum-fee argument. A surrounding application may deduct a fee
before supplying its net contribution, but that fee is outside this algorithm.
Direct transfers that are not recognized as funding do not change $`R`$ or
participants' entitlements. Withdrawal penalties never enter $`R`$.

## Deposits and entry weight

For $`0\le t\lt T`$, the ideal entry weight is

```math
w(t)=1+b\left(1-\frac{t}{T}\right).
```

A positive whole deposit $`a`$ creates a lot. Its actual weight $`w`$ and shares
$`s`$, using the grid-projected $`b`$, are

```math
u=\left\lfloor\frac{T-t}{T}\right\rfloor_\varepsilon,\qquad
w=1+\lfloor bu\rfloor_\varepsilon,\qquad
s=\lfloor aw\rfloor_\varepsilon.
```

Add $`a`$ and $`s`$ to the position and epoch totals. Each lot retains its
shares until a withdrawal removes them or a claim retires them. Entry weight
is non-increasing through the epoch, and rounding can make different times tie.
The host specifies a lot limit per position. The Solana program allows eight
surviving lots per position. Removing a whole lot frees capacity for a deposit.

The deposit action evaluates entry weight at execution time. It has no
minimum-share argument.

## Withdrawals and exit penalty

Withdrawals are accepted only in the live interval and consume the newest
surviving lots first. For a requested whole amount $`x`$, require
$`0\lt x\le\sum_j A_{ij}`$ for the withdrawing position $`i`$. One action may span
several lots. For each visited lot $`h`$, remove
$`x_h=\min(x_{remaining},A_h)`$, where $`A_h`$ and $`S_h`$ are its remaining
principal and shares, then reduce $`x_{remaining}`$ by $`x_h`$.

The shares removed are

```math
\Delta S_h=\begin{cases}
S_h,&x_h=A_h,\\
\min\left(S_h,\left\lceil x_hS_h/A_h\right\rceil_\varepsilon\right),&x_h\lt A_h.
\end{cases}
```

The ideal penalty fraction is $`p(t)=(t/T)^k`$, measured from the epoch start.
For the directed calculation, first set $`v=\lceil t/T\rceil_\varepsilon`$, then
raise $`v`$ to $`k`$ by exponentiation by squaring, rounding each multiplication
upward to the grid, and clamp the result $`p`$ to 1. The returned principal
$`B`$ and forfeited principal $`C`$ are

```math
B=\left\lfloor\lfloor x(1-p)\rfloor_\varepsilon\right\rfloor_{unit},\qquad C=x-B.
```

Remove each lot's $`x_h`$ and $`\Delta S_h`$, discarding fully consumed lots.
Subtract $`x`$ and $`\sum_h\Delta S_h`$ from the epoch totals. Compute $`B`$ and
$`C`$ **once for the entire request**, including a withdrawal spanning multiple
lots. Return $`B`$ to the participant and burn $`C`$ in the same transaction.
Removed shares have no claim. The withdrawal action has no minimum-return
argument; delayed execution uses the penalty at execution time. Destroying
shares increases the remaining positions' fractions of a fixed pot; the
forfeited principal does not increase that pot.

In ideal arithmetic, the newest lot has the lowest or tied entry weight, so
consuming it preserves the most shares for a given withdrawal. Whole-unit
rounding occurs per withdrawal, so dividing withdrawals or deposits among
positions can change the amount returned and shares sacrificed.

## Finalization and claims

At or after the deadline, finalization burns the recorded committed principal
and freezes the surviving shares and tracked net reward. Anyone may submit
the Solana finalization instruction.

```math
S^*=S,\qquad R^*=R,\qquad U=S^*,\qquad V=R^*.
```

$`U`$ tracks unclaimed shares and $`V`$ tracks the remaining net reward.
Reaching the deadline closes the live interval. The separate finalization
action succeeds once and leaves the principal balance as a historical record
of what was burned. When $`R^*=0`$, remaining principal still burns and all
eligible claims pay zero.

If $`S^*=0`$, no position can claim and the entire reward pot remains in escrow.
Otherwise, a position with surviving shares $`S_i\gt 0`$ can claim once, with no
claim deadline:

```math
P_i=\left\lfloor\frac{R^*S_i}{S^*}\right\rfloor_{unit},\qquad U'=U-S_i,\qquad V'=V-P_i.
```

The multiplication and division are performed together with downward rounding,
without first rounding $`S_i/S^*`$ independently. Every claim uses the same
frozen totals. Thus claim order does not change payouts, and a zero-unit claim
still retires the position's exact shares.

The current Solana claim closes the bidder position after transferring its
reward, with authorization from the position's owner. It also closes a
positive-share position whose whole-unit payout is zero. An empty position
cannot use this claim action. No epoch-closure or reward-remainder sweep is
provided. Unclaimed rewards, final rounding remainder, and the whole pot of a
zero-share epoch remain in escrow. No participant receives another participant's
fractional remainder, and no entitlement carries into a later epoch.

## Conservation

For a reachable active epoch, lots add up to the epoch totals and

```math
A=\sum_{i,j} A_{ij},\qquad S=\sum_{i,j} S_{ij},\qquad A\le S\le(1+b)A.
```

The lower bound survives upward rounding of partial share removal. For a lot
with $`S_h\ge A_h`$, its remaining shares are
$`\lfloor S_h(A_h-x_h)/A_h\rfloor_\varepsilon\ge A_h-x_h`$ because the remaining
principal is an exactly representable integer. Deposits establish the bound and
full withdrawals remove both quantities together.

Principal accepted equals principal returned plus withdrawal burns plus
principal burned at finalization. Before finalization, replace its burn term
with the still-committed principal. Principal and reward can be different assets
and are accounted separately.

For a positive-share epoch after all claims, let $`D=R^*-\sum_i P_i`$. Then

```math
D\ge0,\qquad \sum_i P_i+D=R^*.
```

For $`m`$ positive-share positions, $`D\lt m`$ whole reward base units. Across all
accepted funding actions, $`\sum N=\sum_i P_i+D`$. Before all claims complete,
$`V=R^*-\sum_{i\,claimed}P_i`$ includes both outstanding entitlements and final
rounding remainder. In a zero-share epoch, $`V=R^*`$ remains in escrow.
Unrecognized surplus transfers are separate from these tracked amounts.

Splitting a fixed share entitlement across claimants cannot increase its total
payout: the sum of individually rounded-down payouts cannot exceed the payout
obtained by rounding their combined entitlement down once. All positions use the
same frozen ratio, and no last claimant receives the rounding remainder of an
earlier claim.

<!--
title: Revenue Clearing Formal Specification
summary: Mathematical rules for revenue clearing: epoch parameters, funding, weighted deposits, withdrawals, settlement, rounding, and conservation.
tags:
  - revenue-clearing
  - tokenomics
  - math
-->

This note specifies the accounting rules for a
[clearing auction](Revenue-Clearing). Participants commit clearing tokens for
shares of a funded reward pot. Entry time determines the shares created;
withdrawal time determines the forfeiture of principal. Finalization fixes the
reward pot and surviving shares for proportional claims. The
[rationale](Revenue-Clearing-Rationale) explains the purpose of the rules, and
[Time Weight](Revenue-Clearing-Time-Weight) illustrates them.

## Epoch and parameters

An epoch begins at $start$ and lasts $T>0$. Its live interval is
$[start,start+T)$: deposits, withdrawals, and funding are accepted from the
start up to, but not including, the deadline. Write $t=now-start$ for elapsed
time.

Each epoch fixes two parameters:

- **Entry bonus slope** $b$, with $0\le b\le2$. The ideal entry weight starts at
  $1+b$ and declines linearly toward 1. The reference value is $b=1$.
- **Exit penalty exponent** $k\in\{1,2,3,4\}$. It determines the shape of the
  withdrawal penalty. The reference value is $k=2$.

$T$, $b$, and $k$ remain fixed for a given epoch. Subsequent epochs can use
different $b$ and $k$ values. The reference values are examples of parameter
choices.

For a repeating schedule with master start $t_0$ and duration $T$, the epoch
containing $now\ge t_0$ is

$$
n=\left\lfloor\frac{now-t_0}{T}\right\rfloor,\qquad
start_n=t_0+nT,\qquad end_n=start_n+T.
$$

Earlier epochs can be finalized and claimed independently of later epochs. Each
auction has its own pot, positions, and shares.

## Quantities and rounding

During a live epoch, let $A$ be the committed principal, $S$ the surviving
shares, and $R$ the funded net reward. A position holds one or more deposit
lots. Each lot records its remaining principal, remaining shares, and entry
weight.

Principal and rewards are accounted in whole base units of their respective
assets. Shares, weights, and fractions use a fixed-point grid of quantum
$\varepsilon$. Write $\lfloor x\rfloor_\varepsilon$ and
$\lceil x\rceil_\varepsilon$ for rounding down and up to that grid, and
$\lfloor x\rfloor_{unit}$ and $\lceil x\rceil_{unit}$ for rounding to whole
asset units. The selected entry slope is projected downward onto the grid before
use.

Rounding is directed consistently: entry weight and shares round down; the
withdrawal penalty and partial-lot share removal round up; returned principal
and claims round down to whole units; funding fees round up to whole units.
These choices prevent fractional accounting from creating additional whole-unit
claims.

## Funding

A funder adds a gross amount $g$ of the reward asset during the live interval.
For funding fee rate $0\le f\le1$:

$$F=\lceil gf\rceil_{unit},\qquad N=g-F,\qquad R'=R+N.$$

The fee $F$ goes to protocol custody and the net amount $N$ enters the pot. The
rate can differ between funding actions; a funder specifies a maximum acceptable
rate for its action. A change to the fee does not change previously credited
rewards. At a 100% fee the full amount goes to protocol custody. Direct
transfers that are not recognized as funding do not change $R$ or participants'
entitlements.

## Deposits and entry weight

For $0\le t<T$, the ideal entry weight is

$$w(t)=1+b\left(1-\frac{t}{T}\right).$$

A positive whole deposit $a$ creates a lot. Its actual weight $w$ and shares
$s$, using the grid-projected $b$, are

$$
u=\left\lfloor\frac{T-t}{T}\right\rfloor_\varepsilon,\qquad
w=1+\lfloor bu\rfloor_\varepsilon,\qquad
s=\lfloor aw\rfloor_\varepsilon.
$$

Add $a$ and $s$ to the position and epoch totals. A depositor can specify a
minimum acceptable share amount. Each lot retains its entry weight and shares
until a withdrawal removes them or a claim retires them. Weight is
non-increasing through the epoch; rounding can make different entry times tie.

## Withdrawals and exit penalty

Withdrawals are accepted only in the live interval and consume the newest
surviving lot first. For a requested whole amount $x$, require $0<x\le A_h$,
where $A_h$ and $S_h$ are that lot's remaining principal and shares. A
withdrawal across multiple lots consists of successive withdrawals, each against
the newly exposed lot.

The shares removed are

$$
\Delta S=\begin{cases}
S_h,&x=A_h,\\
\min\left(S_h,\left\lceil xS_h/A_h\right\rceil_\varepsilon\right),&x<A_h.
\end{cases}
$$

The ideal penalty fraction is $p(t)=(t/T)^k$, measured from the epoch start. For
the directed calculation, first set $v=\lceil t/T\rceil_\varepsilon$, then raise
$v$ to $k$ by exponentiation by squaring, rounding each multiplication upward to
the grid, and clamp the result $p$ to 1. The returned principal $B$ and
forfeited principal $C$ are

$$B=\left\lfloor\lfloor x(1-p)\rfloor_\varepsilon\right\rfloor_{unit},\qquad C=x-B.$$

Subtract $x$ and $\Delta S$ from the lot, position, and epoch. A fully consumed
lot is removed. The participant can specify a minimum acceptable return.
Forfeited principal is irrecoverable, and removed shares have no claim.
Destroying shares increases the remaining positions' fractions of a fixed pot;
the forfeited principal does not increase that pot.

In ideal arithmetic, the newest lot has the lowest or tied entry weight, so
consuming it preserves the most shares for a given withdrawal. Whole-unit
rounding occurs per withdrawal, so dividing withdrawals or deposits among
positions can change the amount returned and shares sacrificed.

## Finalization and claims

At or after the deadline, finalization irreversibly forfeits all remaining
committed principal and freezes the surviving shares and tracked net reward:

$$S^*=S,\qquad R^*=R,\qquad U=S^*,\qquad V=R^*.$$

$U$ tracks unclaimed shares and $V$ tracks the remaining net reward. Reaching
the deadline closes the live interval; finalization is the separate action that
fixes the totals and forfeits principal. Forfeited tokens can be burned; they
are already irrecoverable before that burn occurs.

If $S^*=0$, the entire reward pot goes to protocol custody. Otherwise, a
position with surviving shares $S_i$ can claim once, with no claim deadline:

$$P_i=\left\lfloor\frac{R^*S_i}{S^*}\right\rfloor_{unit},\qquad U'=U-S_i,\qquad V'=V-P_i.$$

The multiplication and division are performed together with downward rounding,
without first rounding $S_i/S^*$ independently. Every claim uses the same frozen
totals. Thus claim order does not change payouts, and a zero-unit claim still
retires the position's exact shares.

After $U=0$, the finalized epoch can be closed and the remainder transferred to
protocol custody. No participant receives another participant's fractional
remainder, and no fractional entitlement carries into a later epoch.

## Conservation

For a reachable active epoch, lots add up to the epoch totals and

$$A=\sum_{i,j} A_{ij},\qquad S=\sum_{i,j} S_{ij},\qquad A\le S\le(1+b)A.$$

Principal accepted equals principal returned plus withdrawal forfeitures plus
principal forfeited at finalization. Principal and reward can be different
assets and are accounted separately.

For a positive-share epoch after all claims, let $D=R^*-\sum_i P_i$. Then

$$D\ge0,\qquad \sum_i P_i+D=R^*.$$

For $m$ positive-share positions, $D<m$ whole reward base units. Including
funding fees, $\sum g=\sum F+\sum_i P_i+D$. In a zero-share epoch, the entire
net reward goes to protocol custody. Unrecognized surplus transfers are separate
from these tracked amounts.

Splitting a fixed share entitlement across claimants cannot increase its total
payout: the sum of individually rounded-down payouts cannot exceed the payout
obtained by rounding their combined entitlement down once. All positions use the
same frozen ratio, and no last claimant receives the rounding remainder of an
earlier claim.

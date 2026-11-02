<!--
title: Referral Network Formal Specification
summary:
  Core point attribution, voluntary sharing, point-time, reward allocation, and
  conservation for the referral network.
tags:
  - referral
  - math
-->

The [Referral Network](Referral-Network) allocates participation among
depositors and their uplines, then allows members to share part of their
allocation with direct children. This note describes one set of points and one
allocation through a shared tree. An application can apply the same
relationships to separate sets of points and rewards.

## Network and parameters

Participants form a forest of permanent parent-child relationships. A top
participant has depth 0. Each other participant has one parent, and one upline
member at every shallower participating depth. A member's upline is the chain of
members above it; its downline is every member below it. Maximum depth $m$ and
override rate $q$ are fixed for the network:

$$0\le q<1,\qquad mq<1,\qquad 0\le d_i\le m.$$

The reference values $m=2$ and $q=10\%$ permit three participating levels. The
strict product bound makes $1-d_iq$ positive for every permitted depth. Setting
$q=0$ gives no overrides.

For member $i$, let $p_i\ge0$ be its own points, $D_i$ the sum of own points of
its whole downline, and $Z_i=\sum_{c\in children(i)}p_c$ the sum of its
direct children's own points. The total own points are $P=\sum_i p_i$.
Downline and child totals describe the same underlying own points; they are
not additional contributions to $P$.

## Core attribution

Each source at depth $d_i$ assigns an override of $qp_i$ to each upline member
and retains $p_i-d_iqp_i$. Its deposited principal stays with the source. The member's total
core points, including overrides from its whole downline, are

$$b_i=(1-d_iq)p_i+qD_i.$$

Every source's attributed points sum to its own points. Summing over all members
gives

$$\sum_i b_i=\sum_i p_i=P.$$

A member with $p_i=0$ can have $b_i>0$ through its downline. Each upline member
receives $q$ of a source's original points, not $q$ of the amount left after
another upline member has taken an override.

### Example

Let $m=2$ and $q=10\%$. Alice, Bob, and Charlie form a chain, with Bob a child
of Alice and Charlie a child of Bob. Each has 100 own points. In ideal
arithmetic:

| Member           | Own-point retention |  Overrides from downline | Core points |
| ---------------- | ------------------: | -----------------------: | ----------: |
| Alice, depth 0   |                 100 |                       20 |         120 |
| Bob, depth 1     |                  90 |                       10 |         100 |
| Charlie, depth 2 |                  80 |                        0 |          80 |
| **Total**        |                     |                          |     **300** |

If a reward pool allocates 30 units during an interval in which these balances
remain constant, their ideal core earnings are 12, 10, and 8. Each deposit's
gross attribution is an intermediate calculation, not an additional claim
alongside these core earnings.

## Voluntary sharing

A member chooses a sharing fraction $\mu_i\in[0,1]$, initially zero. For a given
period, let $A_i$ be its available allocation: its core allocation for the
period plus gifts received from its parent. If $Z_i>0$, its ideal gift to direct
child $c$ and its own retained amount are

$$G_{i\to c}=\mu_i A_i\frac{p_c}{Z_i},\qquad H_i=(1-\mu_i)A_i.$$

If $Z_i=0$, the member retains all of $A_i$ and makes no gifts. A zero-own-point
child receives no gift from its parent, but may have its own core allocation
from its downline and may share that onward. Children divide gifts by **own
points**, not downline points or the number of children.

A child can include its incoming gift in the amount it shares with its own
children. Gifts flow down the tree; they do not pay overrides
again. For each member, retained allocation plus gifts equals its available
allocation. Consequently, after all sharing, the sum of retained allocations
across the forest equals the sum of the core allocations.

Sharing fractions and own points can change over time. The old balances and
sharing policies determine the allocation up to the change; the new values apply
prospectively. A new or newly eligible deposit does not receive a share of
earlier intervals. When $Z_i=0$, the parent retains that period's allocation;
later children do not receive it retroactively.

## Point-time

Core points accrue with elapsed time. For an interval of duration $\Delta t$
during which points and policies are unchanged, member $i$ has $b_i\Delta t$ of
core point-time before gifts. Point-time can be measured in point-seconds,
point-hours, or another consistent time unit.

An equivalent cumulative formulation accounts for delayed settlement. Let $U_i$
be the cumulative point-time shared by member $i$ per unit of direct-child own
points. After its parent has accounted for the interval, member $i$ has
available point-time

$$A_i=b_i\Delta t+p_i\Delta U_{parent(i)}.$$

For a top participant the incoming term is zero. With fixed-point quantum
$\varepsilon$, the outgoing index increment and retained point-time are

$$
\Delta U_i=\begin{cases}
\left\lfloor A_i\mu_i/Z_i\right\rfloor_\varepsilon,&Z_i>0,\\
0,&Z_i=0,
\end{cases}
\qquad
\Delta H_i=A_i-Z_i\Delta U_i.
$$

Each child is entitled to $p_c\Delta U_i$. Parent retention plus all child
rights equals $A_i$ exactly on the accounting grid. The parent retains the
amount too small to distribute through the index. Per sharing settlement, this
extra retention is less than $Z_i\varepsilon$; it is retained rather than
carried into the next sharing pool. Settlement frequency can therefore affect
the amount shared when arithmetic is rounded.

## From point-time to rewards

Point-time describes participation. An application can attach rewards to that
participation; the reward depends on the reward rate while points are active.
[Emission accounting](Emission-Accounting) explains how Mayflower
combines the network with a time-varying reward stream and keeps the relevant
history.

## Rounding and conservation

The represented override rate and sharing fractions lie on the accounting grid.
The override for each upline member is calculated from the represented $q$ and
the source's original integer points; the source receives the exact complement.
This keeps total core attribution equal to total own points. Sharing uses a
downward-rounded index and gives the donor the exact complement, so retained
amounts plus funded child rights equal the available amount at each split.

The same conservation principle applies to reward sharing. Fixed-point rounding
can leave small amounts with a donor and can make settlement frequency affect
realized sharing. It does not add a second allocation of the underlying reward.
See [Strategic Rationale](Referral-Network-Rationale) for the scope of the
core fixed-capital splitting property.

---
title: Channel Weighting
summary:
  Credit pledges weight the distribution of the common emission stream among
  channels; zero total weight produces an equal split among registered channels.
tags:
  - tokenomics
  - emission
  - referral
---

Channel weighting divides the [[credits-emissions--schedule|scheduled emission]]
among channels. A channel, also called an outlet, corresponds to a market. A
holder pledges existing [[credits|Mayflower credits]] to a channel to influence
the portion of future emission it receives. Pledged credits remain committed
until the owner reduces the pledge; the same credits cannot simultaneously back
another channel pledge. Pledging does not require referral membership.

## Allocation by channel weight

Let channel $i$ have weight $v_i$, equal to the credits pledged to it, and let
$V=\sum_i v_i$. For scheduled emission $E$ over an interval with unchanged
weights, the ideal channel allocation is

$$E_i=E\frac{v_i}{V}\qquad\text{when }V>0.$$

A channel with zero weight receives none of that interval's allocation while
another channel has positive weight. Channel weights choose the destination of
the stream; they do not increase its rate or total.

If $V=0$ and there are $N>0$ registered channels, each receives $E/N$. All
registered channels count, including channels with no eligible points. An empty
channel's allocation is unallocated within that channel, not redistributed to
the others. With no registered channels, the full interval is unallocated.

## A change takes effect immediately

A pledge change settles elapsed emission under the previous weights before
replacing the pledge and the totals. The new weight applies to subsequent
emission, with no waiting period or epoch boundary. An add-and-remove cycle at
the same timestamp earns no elapsed emission.

For example, suppose two channels are registered:

1. With no pledges, each receives half of the stream.
2. A pledge to the first gives it all subsequent emission while the second has
   no weight.
3. An equal pledge to the second changes subsequent shares to one half each.
4. Removing the first pledge gives the second all subsequent emission. Removing
   the last pledge restores the equal split.

A change affects future intervals, not the emission already allocated under
earlier weights. Channel registration follows the same rule: prior emission is
settled first, and a new channel joins the equal split prospectively when all
weights are zero. When weights are positive, a newly registered channel starts
with zero weight and receives no allocation until credits are pledged to it.

## Tracking changing weights

The allocation can also be expressed as two cumulative measures: emission per
unit of positive channel weight and emission per channel during zero-weight
intervals. For an interval with scheduled emission $E$, total weight $V$, and
$N$ channels:

$$
\Delta K=\begin{cases}E/V,&V>0,\\0,&V=0,\end{cases}\qquad
\Delta U=\begin{cases}E/N,&V=0\text{ and }N>0,\\0,&\text{otherwise}.\end{cases}
$$

A channel with unchanged weight $v_i$ over an interval receives
$v_i\Delta K+\Delta U$ in ideal arithmetic. The two measures preserve the
history of switches between weighted and equal allocation; fixed-point routing
rounds and accounts for any unallocated remainder.

## What happens inside a channel?

Each channel has its own eligible points and reward history. Channel weighting
sets how much emission reaches a channel. Once the channel receives an
allocation,
[[credits-emissions--participation|participation and referral rules]] divide it
among members. All channels use the same referral tree within a deployment,
but a member's points and sharing choices in one channel do not become points or
sharing choices in another.

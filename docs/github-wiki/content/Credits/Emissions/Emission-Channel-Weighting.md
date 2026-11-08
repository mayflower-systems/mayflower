<!--
title: Emission Channel Weighting
summary:
  Credit pledges weight the distribution of the common emission stream among
  channels.
tags:
  - tokenomics
  - emission
  - referral
-->

Channel weighting divides the [scheduled emission](Emission-Schedule)
among channels. A channel, also called an outlet, corresponds to a market. A
holder pledges existing [Mayflower credits](Mayflower-Credits) to a channel to influence
the portion of future emission it receives. Pledged credits remain committed
until the owner reduces the pledge; the same credits cannot simultaneously back
another channel pledge. Pledging does not require referral membership.

## Allocation by channel weight

Let channel $i$ have weight $v_i$, equal to the credits pledged to it, and let
$V=\sum_i v_i$. For scheduled emission $E$ over an interval with unchanged
weights, the ideal channel allocation is

$$E_i=E\frac{v_i}{V}\qquad\text{when }V>0.$$

A channel with zero weight receives none of the allocation. Channel weights
choose the destination of the stream; they do not increase its rate or total. A
channel with no eligible points does not pass its allocation to the others; that
amount is unallocated within the channel. If no channel has weight, nothing is
emitted.

## A change takes effect immediately

A pledge change settles elapsed emission under the previous weights before
replacing the pledge and the totals. The new weight applies to subsequent
emission, with no waiting period or epoch boundary. An add-and-remove cycle at
the same timestamp earns no elapsed emission.

For example, suppose two channels are registered:

1. A pledge to the first gives it all subsequent emission while the second has
   no weight.
2. An equal pledge to the second changes subsequent shares to one half each.
3. Removing the first pledge gives the second all subsequent emission.

A change affects future intervals, not the emission already allocated under
earlier weights. A newly registered channel starts with zero weight and
receives no allocation until credits are pledged to it.

## Tracking changing weights

The allocation can also be expressed as one cumulative measure: emission per
unit of channel weight. For an interval with scheduled emission $E$ and total
weight $V$:

$$
\Delta K=\begin{cases}E/V,&V>0,\\0,&V=0.\end{cases}
$$

A channel with unchanged weight $v_i$ over an interval receives $v_i\Delta K$
in ideal arithmetic. The measure preserves the history of weight changes;
fixed-point routing rounds and accounts for any unallocated remainder.

## What happens inside a channel?

Each channel has its own eligible points and reward history. Channel weighting
sets how much emission reaches a channel. Once the channel receives an
allocation,
[participation and referral rules](Participation-and-Referral-Rewards) divide it
among members. All channels use the same referral tree within a deployment,
but a member's points in one channel do not become points in another. A
member's sharing fraction is one choice that applies in every channel.

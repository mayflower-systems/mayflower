<!--
title: Participation and Referral Rewards
summary:
  Eligible collateral supplies channel points; upline overrides and voluntary
  sharing divide each channel's emission into members' retained rewards.
tags:
  - tokenomics
  - emission
  - referral
-->

After [channel weights](Emission-Channel-Weighting) allocate the
common stream among channels, each channel divides its allocation among eligible
participants using the shared [Referral Network](Referral-Network). The
network's tree is shared across channels; points, accumulated rewards, and
mutualization policies are tracked separately for each channel.

## Eligible points

In Mayflower Reserve, a member's deposited AVA collateral contributes one point
per collateral base unit in its market's channel. Each member's own points enter
that channel's total $P$ once. Nonmembers can use the AVM and pledge credits to
channels, but their collateral contributes no emission points. Joining activates
eligible collateral prospectively; it does not earn emission for earlier
intervals.

For a channel allocation $E$ over an interval with unchanged $P>0$, a source
with $p$ own points has an ideal gross attribution

$$G=E\frac{p}{P}.$$

This is the source amount to be divided by referral rules, not a separate payout
to the depositor. If $P=0$, the channel's allocation for that interval is
unallocated; future points cannot claim it. Downline aggregates are used to
determine referral attribution and do not add points to $P$.

## Upline overrides

The network fixes maximum depth $m$ and override rate $q$ with $mq<1$. A
source at depth $d$ assigns an override of $qG$ to each upline member and
retains $(1-dq)G$. Equivalently, a member with own points $p$, downline points
$D$, and depth $d$ has core points

$$b=(1-dq)p+qD.$$

Across the network, core points sum to the original channel points. A member can
earn overrides with no own points. Overrides reallocate the source's
attribution; it does not move its collateral or create additional emission.

For example, with $q=10\%$, maximum depth 2, and Alice → Bob → Charlie each
holding 100 points, their core points are 120, 100, and 80. If the channel
allocates 30 credits while these balances remain constant, their ideal core
earnings are 12, 10, and 8 credits. They do not also claim separate rewards for
their original 100-point deposits.

## Voluntary mutualization

A member can choose a sharing fraction from 0% to 100% for a channel. The
fraction applies to its core earnings plus incoming gifts. Direct children
divide that amount by their own points in that channel, and can share receipts
onward. A zero-own-point child receives no gift, though it can earn from
its downline. When direct children have no own points, the parent retains the
available amount.

Continuing the example, if Alice shares 50% with Bob and Bob shares 25% of his
available earnings with Charlie, final retained rewards are Alice 6, Bob 12, and
Charlie 12 credits. The total is the same 30 credits.
[Mutualization](Mutualization) explains the calculation and
its rounding.

Points and sharing choices are settled under their old values before a change.
The new values apply only to subsequent accrual. A member's final retained
credit—not its gross source attribution or the amount it shares away—is the
amount it can collect.

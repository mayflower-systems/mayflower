<!--
title: Referral Network
summary:
  A depth-limited network that divides participation points and rewards between
  depositors and their uplines, with optional sharing back to children.
tags:
  - referral
  - tokenomics
-->

The Referral Network rewards both participation and productive recruitment. Each
participant has a position in a referral tree and can contribute points through
eligible deposits. A deposit's points are divided between the depositor and its
upline: each upline member receives the same fixed fraction, and the depositor
retains the rest. A participant can also choose to share part of its core
earnings with its direct children. These rules redistribute a fixed reward
allocation; they do not create additional points or emission.

In Mayflower, the network divides a channel's allocation of
[Mayflower credits](Mayflower-Credits) among members. Eligible deposited collateral
supplies points, and [channel weights](Emission-Channel-Weighting)
direct the emission budget among outlets. The referral network determines how an
outlet's allocation is divided, not the size of the overall emission schedule.

## The pieces

- **Member, upline, and downline:** a member occupies a position in a tree. A
  member's parent is the member whose offer it joined. Its upline is its parent,
  its parent's parent, and so on; its downline is every member below it. An
  accepted upline is permanent. A network may contain multiple trees.
- **Depth:** a top participant has depth 0, its child has depth 1, and so on.
  The maximum depth $m$ limits how long a participant's upline can be.
- **Own points:** a member's eligible deposit contributes points. The total $P$
  counts each member's own points once.
- **Override:** each source assigns a fixed fraction $q$ of its own points, the
  override rate, to **each** upline member. The source retains the remainder.
  Overrides change reward attribution; they do not transfer the deposited
  principal.
- **Core points:** the points a member retains from its own deposit plus
  overrides from its whole downline. These determine the member's allocation before
  voluntary sharing.
- **Mutualization:** a member can share a chosen fraction of its core
  earnings with direct children, in proportion to their own points. Children keep
  these gifts and do not share them onward.

## How core allocation works

For a member at depth $d$ with $p$ own points and $D$ own points across its
whole downline, its core points are

$$b=(1-dq)p+qD.$$

The network fixes maximum depth $m$ and override rate $q$, with $0\le q<1$ and $mq<1$.
The last condition keeps the own-point retention fraction positive at every
permitted depth. A downline member's contribution to $D$ is its own points, counted
once; $D$ is not an extra addition to the total own points.

For example, with $m=2$ and $q=10\%$, a deposit of 100 points at depth 2 assigns
10 points to each of its two upline members and retains 80. A deposit of 100 at depth
1 assigns 10 to its parent and retains 90; at depth 0 it retains all 100. A
member can earn overrides from its downline even if it has no deposit of its
own.

Each source's retained points and overrides sum to its own points.
Therefore the total core points across the network equal the total own points.
For a reward $E$ attributed over an interval with $P>0$ unchanged own points, a
member's ideal core reward is $E\,b/P$. Voluntary sharing then redistributes
those earnings downward. [Credit Emissions](Credit-Emissions) explains how
Mayflower supplies and distributes rewards.

## How voluntary sharing works

Each member chooses a sharing fraction from 0% to 100%, initially 0%. The
fraction applies only to its own core earnings. Direct children divide the shared
amount according to their own points. A child keeps its whole gift and does not
share it onward, so a gift passes down exactly one level. Gifts do not pay
overrides again.

A child with zero own points receives no gift from its parent, even if it earns
overrides from its downline. If all direct children have zero own points, the
parent retains its core earnings. Sharing fractions can change for future
earnings, from each channel's next settlement; earlier earnings remain
attributed under the earlier policy and the point balances in effect when they
accrued.

## Why use a network?

Overrides reward a member for bringing in a productive downline, rather than
merely creating accounts. The same fraction of a source's points reaches each
upline member, so a grandparent's override does not depend on the intervening
parent's own deposit. Shallower positions retain more of their own attribution
and have more remaining recruiting depth.

Mutualization lets a parent make its position more attractive to children by
sharing some of its core earnings. The amount a child receives depends on the
parent's core earnings, its sharing fraction, and the points of the other direct
children. These economic choices are explained in
[Strategic Rationale](Referral-Network-Rationale) and
[Link Shopping](Link-Shopping).

## Notes

- [Formal Specification](Referral-Network-Formal-Specification) — Core attribution,
  point-time, reward allocation, rounding, and conservation.
- [Mutualization](Mutualization) — Downward sharing and a
  worked example.
- [Strategic Rationale](Referral-Network-Rationale) — Recruitment, depth, and
  the economics of splitting positions.
- [Link Shopping](Link-Shopping) — Joining offers, fees, and
  how to compare permanent positions.
- [Protocol Parameters](Referral-Network-Parameters) — Fixed network
  rules and node owners' joining and sharing choices.
- [Diagrams](Referral-Network-Diagrams) — The flows of points, rewards, and
  gifts.

## How it connects

Membership is required for collateral to count as emission points. Joining
activates eligible points prospectively; it does not create rewards for earlier
periods. Using the AVM and pledging credits to channels do not require referral
membership. The [emission system](Credit-Emissions) uses the same tree
across channels, with separate points, earnings, and gifts for each channel. A
member's one sharing fraction applies in every channel.
Earned credits can be pledged to channels or used in
[revenue clearing](Revenue-Clearing).

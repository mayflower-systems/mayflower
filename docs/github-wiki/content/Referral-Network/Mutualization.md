<!--
title: Mutualization
summary:
  How members voluntarily share their core earnings with direct children, in
  proportion to those children's own points.
tags:
  - referral
  - tokenomics
-->

Mutualization is voluntary sharing down a [referral tree](Referral-Network).
After core attribution assigns a member earnings from its own points and
downline, the member can share a chosen fraction of those core earnings with
its direct children. A gift received from the member's parent is kept whole and
is never shared onward, so a gift passes down exactly one level.

## How a gift is divided

Let a parent have core earnings $E$, sharing fraction $\mu$, and direct
children with own points $p_1,p_2,\ldots$. Let $Z=\sum_c p_c$. When $Z>0$, child
$c$ receives, in ideal arithmetic,

$$G_c=\mu E\frac{p_c}{Z}.$$

The parent retains $(1-\mu)E$, plus the whole of any gift from its own parent.
Children's **own** points determine their proportions; their downlines' points
do not enter this particular split. A
child with zero own points receives no parent gift, although its downline can
still generate core earnings for it. If $Z=0$, the parent retains all its
earnings.

The parent's gift is paid from its own core earnings. Sharing changes who
retains existing earnings; it does not increase the reward budget. A gift goes
downward and does not pay overrides a second time.

## Example: sharing through two levels

Suppose Alice → Bob → Charlie each have 100 own points, with $q=10\%$ and
maximum depth 2. Their core points are 120, 100, and 80. In a period with 30
reward units and no changes to points, their ideal core earnings are 12, 10,
and 8.

1. Alice chooses to share 50%. Bob is her only direct child with own points, so
   Alice retains 6 and gives Bob 6.
2. Bob chooses to share 25% of his 10 core earnings. Charlie is his only direct
   child with own points, so Bob gives Charlie 2.5. Bob keeps the other 7.5 and
   Alice's gift of 6 whole, so he retains 13.5.
3. Charlie retains 10.5: 8 core earnings plus Bob's gift of 2.5. Bob's gift came
   from his own core earnings, not from Alice's gift. The final retained
   earnings are Alice 6, Bob 13.5, and Charlie 10.5.

The three retained amounts sum to the original 30 reward units. If Alice has
another direct child with the same own points as Bob, her gift is divided
equally between those two children instead. Their downlines do not change
their proportions of that particular gift.

## Sharing offers change over time

Each node owner chooses one sharing fraction, and it applies in every emission
channel. In each channel, the parent shares that fraction of what it earned in
that channel, and its children divide the gift by their own points in that
channel. The credits a member keeps from every channel gather in one balance.
The fraction starts at zero and can be changed for future earnings. A change
applies in each channel from that channel's next settlement; earnings accrued
before then use the earlier fraction. A new child or newly activated deposit
participates from that point forward; it does not receive earlier gifts. If a
parent had no direct children with positive own points during an interval, it
retains that interval's core earnings.

A sharing percentage alone does not determine a child's reward. The parent's
core earnings set the size of the pool, and all direct children's own
points determine how that pool is divided. A parent can have children it
controls itself; their points participate in the same division and reduce the
fraction available to other children.

## Point-time and reward time

The same sharing pattern can describe participation as **point-time** or
allocate a funded reward stream. Point-time measures points held over time.
Reward earnings also depend on the reward rate and total points while that
participation occurs, so equal point-time can receive different rewards when
those quantities change.

Sharing is accounted with directed rounding. A parent retains any amount too
small to divide among child points on the accounting grid. The
[formal specification](Referral-Network-Formal-Specification) defines the equations,
timing, and conservation rules.

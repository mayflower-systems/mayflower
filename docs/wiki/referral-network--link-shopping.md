---
title: Link Shopping
summary:
  How depth, recruiting room, admission terms, and voluntary sharing affect the
  value of a referral position.
tags:
  - referral
  - marketplace
---

A referral offer gives a prospective member a position under a particular parent
in the [[referral-network|referral tree]]. That position determines the member's
depth and upline. Once accepted, its upline is permanent. Comparing offers
means comparing both the position and the parent's voluntary sharing policy.

## Admission offers

A node owner can publish one active offer for new members to join directly
beneath that node, provided another participating level is available. The offer
specifies a current joining fee and a nonnegative increment. A successful join
pays the current fee to the parent node's owner, then increases that offer's fee
by the increment for the next join. A zero fee or zero increment is possible.

For example, an offer with a fee of 10 units and an increment of 2 charges
successive joiners 10, 12, and 14 units if its owner makes no intervening
change. A prospective member specifies the maximum fee it is willing to pay when
joining; the join fails if the current fee exceeds that maximum.

The node owner can change the fee or increment for future joins, close the
offer, or replace it with a new one. These actions do not change the upline of
members who have already joined. The admission fee is separate from the member's
points, overrides, and voluntary sharing: paying it does not create an
additional emission allocation.

## Position and depth

At depth $d$ with override rate $q$, a member retains $1-dq$ of the attribution from
its own points. Each upline member receives $q$ of those points. A shallower position
retains more of its own attribution and has more remaining depth in which to
recruit a downline.

For example, with maximum depth 2 and an override rate of 10%, a top member retains 100% of
its own-point attribution, a member at depth 1 retains 90%, and a member at
depth 2 retains 80%. A depth-1 member can earn overrides from a child at depth 2.
A depth-2 member has no remaining participating depth for children.

Depth is one part of the offer. Availability and the admission price depend on
the active offer for that position. A member may have a position with no own
points and earn from a productive downline, but its position and the downline's
deposits determine those earnings.

## Parent's sharing offer

A parent may share a fraction of its available earnings with its direct
children. In ideal arithmetic, a child with own points $p$ receives

$$\text{gift}=\mu A\frac{p}{Z},$$

where $\mu$ is the parent's sharing fraction, $A$ is the parent's available
earnings for the period, and $Z$ is the sum of the direct children's own points.
The available amount includes the parent's core earnings and gifts it receives
from above.

A higher sharing fraction allocates more of a given pool to children, but a
percentage alone does not determine the amount a child receives. The pool can
change as points and rewards change; new or larger child deposits also change
the division. A child with zero own points receives no parent gift. If every
direct child has zero own points, the parent keeps the pool.

A parent's own children can include positions it controls. Those points share in
the pool and reduce the fraction received by the other children. The accepted
parent relationship is permanent, while the parent's sharing fraction can be
changed for future earnings; an offer's current sharing fraction is not a
permanent rate.

## Compare the whole offer

A prospective member can compare:

1. **Depth:** how much of its own-point attribution it retains and how much
   recruiting depth remains.
2. **Admission terms:** the current joining fee and how it changes for
   subsequent joins.
3. **Parent's sharing:** the current fraction and the earnings from which the
   parent shares.
4. **Other children:** the own points that divide the parent's sharing pool in
   the relevant participation set.
5. **Future participation:** the points the member and its downline may
   contribute and the reward being allocated while those points are active.

These factors interact. A shallower position has a stronger core allocation for
the same own points, while an active sharing policy can add gifts to a deeper
member's earnings. [[referral-network--mutualization|Mutualization]] gives a
worked example, and [[referral-network--rationale|Strategic Rationale]] explains
the incentives.

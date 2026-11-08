<!--
title: Link Shopping
summary:
  How depth, recruiting room, admission terms, and voluntary sharing affect the
  value of a referral position.
tags:
  - referral
  - marketplace
-->

A referral offer gives a prospective member a position under a particular parent
in the [referral tree](Referral-Network). That position determines the member's
depth and upline. Once accepted, its upline is permanent. Comparing offers
means comparing both the position and the parent's voluntary sharing policy.

## Admission offers

Every node has an open offer for new members to join directly beneath it,
provided another participating level is available. The offer specifies a current
joining fee and a nonnegative increment. A successful join pays the current fee
to the parent node's owner, then increases the fee by the increment for the next
join. A new member sets the offer of its own node when it joins.

A top-level seat has no parent. The protocol sells top-level seats under its own
offer, which works the same way: each join pays the current fee to the protocol,
then raises the fee by the increment. The protocol can change that offer at any
time.

For example, an offer with a fee of 10 units and an increment of 2 charges
successive joiners 10, 12, and 14 units if its owner makes no intervening
change. A prospective member specifies the maximum fee it is willing to pay when
joining; the join fails if the current fee exceeds that maximum.

The node owner can change the fee and the increment at any time. A change
applies to future joins and does not change the upline of members who have
already joined. The admission fee is separate from the member's points,
overrides, and voluntary sharing: paying it does not create an additional
emission allocation.

## Position and depth

At depth $d$ with override rate $q$, a member retains $1-dq$ of the attribution from
its own points. Each upline member receives $q$ of those points. A shallower position
retains more of its own attribution and has more remaining depth in which to
recruit a downline.

For example, with maximum depth 2 and an override rate of 10%, a top member retains 100% of
its own-point attribution, a member at depth 1 retains 90%, and a member at
depth 2 retains 80%. A depth-1 member can earn overrides from a child at depth 2.
A depth-2 member has no remaining participating depth for children.

Depth is one part of the offer, and the parent's current fee is another. A
member may have a position with no own points and earn from a productive
downline, but its position and the downline's deposits determine those earnings.

## Parent's sharing offer

A parent may share a fraction of its core earnings with its direct
children. In ideal arithmetic, a child with own points $p$ receives

$$\text{gift}=\mu E\frac{p}{Z},$$

where $\mu$ is the parent's sharing fraction, $E$ is the parent's core
earnings for the period, and $Z$ is the sum of the direct children's own points.
Gifts the parent receives from above are not part of $E$; the parent keeps them.
A child likewise keeps its whole gift and does not share it onward.

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
member's earnings. [Mutualization](Mutualization) gives a
worked example, and [Strategic Rationale](Referral-Network-Rationale) explains
the incentives.

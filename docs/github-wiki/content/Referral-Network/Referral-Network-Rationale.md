<!--
title: Referral Network Rationale
summary:
  Why the referral network rewards productive recruitment, how depth and
  voluntary sharing create different offers, and how splitting positions affects
  core attribution.
tags:
  - referral
  - design
-->

The [Referral Network](Referral-Network) connects a member's reward to two
sources: the member's own eligible points and points contributed by its
downline. Fixed upline overrides reward recruitment without enlarging the
underlying reward budget. Voluntary
[mutualization](Mutualization) lets members share part of
their earnings with the children who help produce them.

## Reward productive recruitment

Each source assigns the same fraction $q$ of its own points to each upline
member. An upline member's override therefore depends on downline points, rather
than on how many accounts its downline creates. A member can earn from a
productive child or grandchild without making a deposit of its own. The override
is calculated from the source's original points at every level, so a
grandparent's fraction does not shrink because an intervening parent has few
points.

For example, with a 10% override rate and maximum depth 2, Alice has 100 own points, Bob
has none, and Bob recruits Charlie with 1,000. Their core points are Alice 200,
Bob 100, and Charlie 800: Alice retains her 100 and receives 100 from Charlie,
Bob earns 100 from Charlie, and Charlie retains 800. The total remains 1,100. If
Bob shares half of his earnings and Charlie is his only direct child with
positive own points, Bob retains half and Charlie receives the other half.

## Each layer has a different value

A member at depth $d$ retains $1-dq$ of its own-point attribution. With $q=10\%$
and maximum depth 2, the fractions are 100%, 90%, and 80% at depths 0, 1, and 2.
A shallower position also has more remaining depth in which to recruit
a downline. Maximum depth determines how many layers can receive overrides from
any source; it does not by itself limit the number of top positions.

Recruiting changes both a member's core points and total own points. A larger
raw override does not automatically mean a larger share of a fixed reward. For
example, Alice has 100 own points and is the only member in the allocation, so
she initially has all 100 core points out of 100. A new member with 100 points
joins directly below her at $q=10\%$: Alice now has 110 core points out of 200,
or 55%. If the same 100 points instead arrived in a separate tree, Alice would
have 100 out of 200, or 50%. The referral gives Alice a larger share than that
alternative, while the new points dilute her original share.

## Splitting existing capital

Consider a controlled subtree whose top member is at depth $d$. If its members
have total own points $S$, its combined core points are

$$(1-dq)S.$$

Each source in the subtree gives the same override to each outside upline
member, regardless of where its points sit inside the subtree. Overrides to
controlled upline members remain within the group. Moving fixed points among
controlled downline members under an unchanged outside upline therefore leaves
their combined **core** attribution unchanged. The same reasoning applies to
splitting points among siblings with the same outside upline.

The property concerns core attribution from existing points. Voluntary gifts use
a separate rule: a parent can share with children it controls, and those
children's own points affect how gifts are divided. Creating a controlled
intermediary above a newly recruited outside member is also a different
arrangement. At $q=10\%$, inserting an intermediary between a top member and a
new recruit gives that intermediary 10% of the recruit's points and reduces the
recruit's own retention from 90% to 80%; it does not create extra points.
An accepted upline is permanent, so this example concerns a new recruit's chosen
path.

## Why share earnings with children?

A parent can offer mutualization to attract or retain productive children. The
sharing fraction alone is not the whole offer. A child's share also depends on
the parent's available earnings, the child's own points, and the own points of
the parent's other direct children. A parent can itself receive gifts from
above, which increase the available amount it can share onward.

Children a parent controls participate in the same pro-rata division as other
children. Their presence reduces each other child's fraction of the pool.
Sharing policies can change prospectively, while the accepted parent
relationship remains fixed. [Link Shopping](Link-Shopping)
explains how these terms affect a prospective member's choice.

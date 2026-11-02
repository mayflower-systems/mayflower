<!--
title: Emission Accounting
summary:
  How changing channel allocations and points become retained referral credit,
  when credits are minted, and how the emission budget is conserved.
tags:
  - tokenomics
  - emission
  - math
-->

[Emissions](Credit-Emissions) accrue over time before they become minted
credits. The schedule establishes a budget, channel weights route it to
channels, and each channel attributes its allocation to eligible points and
referral members. The resulting retained reward entitlement becomes circulating
credits when a member collects it.

## Accounting through time

For a channel with allocated emission rate $e(t)$ and eligible own-point total
$P(t)>0$, the ideal cumulative reward per point is

$$J(t)=\int\frac{e(t)}{P(t)}\,dt.$$

A member's core points, which already include overrides, earn according
to the change in $J$ while those points are active.
[Mutualization](Mutualization) can share the resulting
earnings down to children, who can share receipts onward. A member's reward
entitlement is what it retains after sharing.

Time matters. Suppose a channel has 200 points throughout two one-hour periods
and receives 10 credits in the first hour and 90 in the second. A position with
100 core points only in the first period earns 5 credits; one with 100 core
points only in the second earns 45 credits. Both contribute 100 point-hours, but
their rewards differ. Point-time and a single average rate cannot reconstruct
the reward history when participation and emission vary independently.

The accounting preserves this history for rewards and for gifts passed to
children. Channel pledges, eligible points, and sharing policies are applied to
the intervals in which they were in force. A change first settles the preceding
interval, then takes effect prospectively. This keeps newly registered channels,
new members, new deposits, and new policies from claiming earlier allocations.

## Applying referral sharing to rewards

For an interval in which balances and policies are unchanged, let $b_i$ be
member $i$'s core points, $p_i$ its own points, and $\Delta J$ the channel's
reward-per-point increment. Let $\Delta T_{parent(i)}$ be the reward shared by
its parent per unit of direct-child own points. The member's available reward
is, in ideal arithmetic,

$$A_i=b_i\Delta J+p_i\Delta T_{parent(i)}.$$

For a top member, the incoming term is zero. If the member shares fraction
$\mu_i$ and its direct children have total own points $Z_i>0$, its outgoing
reward per child point is $\Delta T_i=\mu_i A_i/Z_i$; otherwise it shares
nothing. Its retained reward is $A_i-Z_i\Delta T_i$. Children receive their own
points multiplied by that outgoing index and can share their receipts onward.
This is the referral rule applied to actual reward history, separately from
ordinary point-time.

## Point indexes and residuals

For a newly realized channel allocation $Y$, existing point-index remainder
$\rho$, and $P>0$ own points, the fixed-point increment is

$$\Delta J=\left\lfloor\frac{\rho+Y}{P}\right\rfloor,\qquad \rho'=\rho+Y-P\Delta J.$$

The remainder stays with the channel's point accounting while the denominator is
unchanged. When the point contribution changes, the old denominator's remainder
is recorded as unallocated before the new denominator takes effect. When $P=0$,
realized allocation is unallocated instead of entering a point index. Neither
amount is passed to later arrivals.

Channel routing similarly records the difference between emission and the
amounts representably routed as unallocated. Referral sharing rounds the gift
index down and leaves its undistributable remainder with the sharing member.
Base reward accrual also rounds down to the accounting grid. These rounding
rules determine how finite-precision amounts are attributed without creating an
extra claim.

## Collection and supply

A member can collect its final retained reward entitlement for a channel in
whole credit base units. Collection debits that entitlement and issues the
corresponding credits. Fractional entitlement remains accounted for until enough
is available to collect a whole unit. A source's gross point attribution is not
an independent claim: the one claim is the retained referral entitlement after
overrides and mutualization.

A deployment may have an existing bootstrap supply before mint authority
transfers to Reserve. That existing supply is distinct from scheduled emission.
Once Reserve controls the mint, ongoing emission enters supply through
successful collection; recording elapsed schedule entitlement alone does not
issue credits. Credits pledged to a channel were already issued and are returned
when the owner reduces the pledge.

## One budget, several allocation steps

The schedule's accrued entitlement is divided into routed and unallocated
amounts. Each channel's realized allocation is divided among indexed
participation, unallocated amounts, and its point-index remainder. Referral
overrides and gifts divide funded earnings further; they do not create another
budget. Collecting a retained entitlement converts an earned claim into issued
credits rather than creating a second claim against that same allocation.

This separation also explains supply over time: an uncollected reward
entitlement has accrued but has not yet been issued; unallocated emission is not
reassigned to later participants; and credits irreversibly spent through
[revenue clearing](Revenue-Clearing) are separate from the schedule.

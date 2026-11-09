<!--
title: Emission Channel Weighting
summary:
  An independent allocator routes the declining emission stream to asset lanes,
  preserving allocation history and explicitly accounting for unallocated amounts.
tags:
  - tokenomics
  - emission
  - referral
-->

Channel weighting decides which asset lanes receive the
[scheduled emission budget](Emission-Schedule). A lane, also called a channel,
has one asset identity, its own eligible balance total, and its own reward
history. The [referral specification](Referral-Network-Formal-Specification)
treats allocation as an independent layer. The proportional gauge rule below is
the allocation mechanism used by the credit-pledge design; voting rights and
custody remain distinct from the core referral calculation.

## The allocator's contract

Let $`E(t)`$ be the continuously declining global emission rate and $`e_l(t)`$
the rate assigned to lane $`l`$. In ordinary active operation the allocator
routes the full budget:

```math
\sum_l e_l(t)=E(t).
```

One active lane with no gauge competition receives $`E(t)`$. Adding a lane does
not create another emission stream. The allocator must supply authenticated,
chronological allocations; a new weight affects future accrual after the
previous interval has been settled.

If routing is unavailable, the amount is explicitly unallocated and unissued.
Numerical routing residuals must also be accounted for. These exceptions are not
a discretionary reserve that the allocator may silently retain from otherwise
valid active allocations.

An allocation to a lane with no eligible assets is likewise unallocated for that
interval. It does not pass automatically to other lanes or become a reward for a
later entrant. An allocator may choose a future allocation policy that avoids
empty lanes, provided that policy is explicit and preserves the global budget.

## A proportional gauge adapter

For an adapter using nonnegative lane weights $`v_l`$ with total
$`V=\sum_l v_l\gt 0`$, a conventional proportional rule is

```math
e_l(t)=E(t)\frac{v_l}{V}.
```

For example, unchanged weights 2 and 1 divide an interval's integrated
300-credit budget into 200 and 100, even while its emission rate declines. Equal
weights would divide that same interval budget into 150 per lane. A later weight
change applies to later scheduled budgets; it does not rewrite the earlier
allocation.

The formula alone does not settle how voting power is acquired or held.
[Credit Channel Weighting](Credit-Channel-Weighting) specifies the credit-pledge
policy and exclusive custody. If such an adapter has no valid allocation because
$`V=0`$, it must report the unavailable routing budget rather than quietly
change the global emission schedule or give a later voter the backlog.

An adapter may preserve changing weights through a cumulative index such as

```math
K(t)=\int_{t_0}^{t}\mathbf{1}_{V(u)\gt 0}\frac{E(u)}{V(u)}\,du,
```

where the integrand is defined as zero when $`V(u)=0`$; that interval's
scheduled budget is recorded separately as unallocated.

A lane whose weight stays constant during an interval receives $`v_l\Delta K`$.
The changing emission rate is integrated over time; it does not require a new
transaction at every instant. Weight changes must checkpoint the old interval
before taking effect. An implementation cannot multiply today's weight by an
entire unprocessed past interval and call that historical allocation.

## Asset units stay inside their lanes

For lane $`l`$, the core referral denominator is $`P_l=\sum_i p_{i,l}`$, the sum
of members' own eligible balances in that asset. The system never adds raw BTC,
ETH, or other asset quantities into one balance denominator. Gauge allocation
expresses the chosen relative earning power by routing credit budgets; it does
not make unlike asset units interchangeable.

All lanes use the same permanent referral connections and the same fixed
commission geometry.
[Participation and Referral Rewards](Participation-and-Referral-Rewards) divides
each lane's assigned credits among asset holders and their ancestors.

## Common family marketing is a separate allocation

After core referral allocation, a parent can share a fraction of its gross
commissions from all lanes. Children divide that common pool by their escrowed
credits, not by the parent's gauge weights or by their own BTC or ETH balances.
A child with no holdings in a particular lane can therefore receive marketing
funded by that lane.

Gauge commitments and family escrow are separate uses of already-issued credits.
The reference custody model does not let the same principal simultaneously fund
both, or a clearing commitment or auction bid. Any future governance rights over
family escrow need an explicit integration rule; they must not duplicate reward
weight or keep spent principal active.

Changing a family's escrow or marketing percentage requires the parent to settle
all of its contributing lane sources at one effective time. The allocator's
chronological history makes that possible, but an allocation index alone does
not perform the family checkpoint. See
[Emission Accounting](Emission-Accounting) and
[Credit Channel Weighting](Credit-Channel-Weighting).

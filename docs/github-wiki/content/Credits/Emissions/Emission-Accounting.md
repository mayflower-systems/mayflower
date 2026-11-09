<!--
title: Emission Accounting
summary:
  Lane and family indices preserve historical ownership across asset trades,
  credit escrow changes, collection, and multi-lane settlement barriers.
tags:
  - tokenomics
  - emission
  - math
-->

[Emissions](Credit-Emissions) accrue as earned entitlement before they become
issued credits. This page describes the accounting in the
[referral specification](https://github.com/mayflower-systems/mayflower/blob/a1f87bfa8469630444626ddbc11536512c78bb16/referral/rust/mayf-referral/SPEC.md).
The formulas below are the exact reference model. The kernel uses cumulative
indices and explicit fractional carries; its source documentation specifies
rounding ownership and the host integration contract.

## Lane history and core accrual

Let $E(t)$ be the continuously declining global rate and $e_l(t)$ the
authenticated rate assigned to lane $l$. Let $P_l(t)$ be that lane's total own
eligible asset balance. Its ideal cumulative reward per asset unit is

$$I_l(t)=\int_{P_l(t)>0}\frac{e_l(t)}{P_l(t)}\,dt.$$

Allocation to an empty lane is unallocated and unissued, not a backlog for its
next depositor. The external allocator must preserve previous weight intervals;
using today's weight for an entire unprocessed past interval is incorrect.

The tree fixes maximum earning depth $H$ and ancestor commission $q$, with
$q>0$ and $Hq<1$. Auctioned seats have depth zero. For a member whose own balance
$p_{i,l}$ and strict descendant balance $D_{i,l}$ remain constant between lane
checkpoints, an index increment $\Delta I_l$ produces

$$\Delta A_{i,l}=(1-d_iq)p_{i,l}\Delta I_l,\qquad
\Delta O_{i,l}=qD_{i,l}\Delta I_l.$$

The first term is own-asset income; the second is gross referral commission.
Together they are the member's core earnings from that interval. Raw asset units
are combined only within their lane; the resulting credit amounts can be added
across lanes.

Both emission history and allocation history matter. Even while the global rate
declines, a lane with 200 asset units might receive 10 credits in one hour and 90
in the next because its share of that stream increases. In that example,
100 effective core units held only during the first hour earn 5 credits. The
same units held only during the second earn 45. A single average rate over both
hours would lose that history.

Continuous rate variation is integrated into the cumulative lane index. It does
not require a transaction at every instant. Checkpoints at balance, allocation,
escrow, or policy changes preserve the relevant intervals.

## One family index across lanes

For parent $i$, let $s_c$ be each immediate child's issued-credit escrow and
$S_i=\sum_c s_c$. The parent's discretionary marketing fraction is $\mu_i$.
Its cumulative family index $F_i$ measures marketing credits allocated per
escrowed credit, not per BTC or ETH unit.

For a valid member/lane settlement under unchanged family state:

```
own      = (1 - d_i*q) * p_i,l * delta_I_l
override = q * D_i,l * delta_I_l

if S_i > 0:
    marketing = mu_i * override
    F_i += marketing / S_i
else:
    marketing = 0

kept_i += own + override - marketing
I_i,l_seen = I_l
```

Only gross overrides fund marketing. Own-asset earnings, fees, escrow principal,
and incoming gifts do not enter that source. If no child escrows credits, the
parent keeps the otherwise undistributable amount.

A child records its parent's family index at its previous incoming checkpoint:

```
kept_c += s_c * (F_parent - f_c_seen)
f_c_seen = F_parent
```

This gift does not update the child's own $F_c$. Gifts are terminal, and parents
cannot also collect the gross commissions already assigned to their children.
An unchanged child balance can remain lazy while the parent's index advances;
siblings do not all need to be loaded for each settlement.

## Frequent asset changes

A buy, sale, deposit, withdrawal, or other authenticated eligible-position change
first advances the affected lane to the event time under its old total. It then
settles the affected path from the earning seat toward the member under the old
balances, before updating the member's own balance, every ancestor's descendant
total, and $P_l$.

This preserves earnings before the trade and applies new balances only to future
accrual. Credit escrow does not change automatically with an asset trade, and
trading assets never changes the permanent parent connection.

## The all-source family barrier

**Before changing a parent's marketing fraction, or any child's escrow and hence
the family's total escrow, settle every contributing lane of that parent to the
same effective time using the old family state.**

For an escrow change, then settle the affected child's incoming gift using its
old escrow amount. Only after that checkpoint may principal and the child/family
escrow counters change. A marketing change likewise takes effect immediately
after the common checkpoint, for subsequent accrual in every lane.

The source set must be complete and authenticated. A caller cannot supply only
convenient lanes. Track historical member/lane records that can contain pending
income; a current zero asset balance does not justify forgetting their history.
Since incoming gifts never fund outgoing marketing, this parent-source barrier
does not require recursively materializing every ancestor's marketing income.

Consider two lanes:

1. A escrows 100 credits and B escrows none.
2. BTC and ETH each accrue 100 marketing credits. Before B enters, both sources
   settle under the old stake; A owns all 200.
3. B escrows 100 credits. In the next interval, each lane produces 50 marketing
   credits. A and B each receive 50 from this new 100.

Correct cumulative family earnings are A = 250 and B = 50. If BTC settles before
B enters but stale ETH income later uses the new denominator, B incorrectly
captures 50 credits from before its deposit. A single aggregate family index
does not prevent this error without the complete barrier.

Withdrawals and marketing-rate changes require the same protection. The first
staker receives no backpay from intervals with $S_i=0$; the last unstaker keeps
its earned claim. A stake-and-unstake cycle at one timestamp earns no intervening
reward.

## Initial state and complete collection

A new member starts with zero own balances, descendant balances, credit escrow,
child escrow total, family index, marketing fraction, and accrued entitlement.
An ordinary child's incoming checkpoint starts at its parent's stored $F$; a
top-level seat has no incoming family claim. The first positive credit stake
still performs the all-source barrier before it becomes active.

A missing member/lane record represents zero contribution before activation.
Checkpoint the current lane index before adding its first nonzero own or
descendant balance, including an ancestor's first activation. A zero-escrow
member still advances its incoming checkpoint when settled.

Collection can mint already-accrued net entitlement without eagerly processing
every source. An operation advertised as collecting everything through time $t$
must first settle the member's own core sources and its immediate parent's
gross-override sources, then advance the incoming family checkpoint. Iterating only the
claimant's own asset lanes is insufficient: a member with no asset records can
still earn gifts funded by every lane of its parent.

Collection debits the earned balance and issues the same whole credit amount
once. Fractional earned entitlement remains with that member. A second
collection without new entitlement mints nothing. All required state updates,
custody movements, and mint/burn effects must succeed or fail together.

## Execution cost

A single-lane asset change requires work proportional to affected ancestry. A
credit-escrow or marketing change also processes all contributing lane sources
of the affected parent in the reference implementation. It does not scan every
sibling, but its work is not independent of the number of lanes.

The implementation must demonstrate that the complete barrier fits its
execution limits or prove equivalent historical/versioned accounting. Splitting
the barrier into transactions requires explicit snapshot, concurrency, and commit
rules; partially processed history must not activate new stake against old
income. The target design does not introduce an arbitrary lane cap, epoch
duration, or economic lock period to conceal this requirement.

## Finite precision and residual ownership

The implementation must define numeric bounds, rounding, and remainder ownership.
Independently flooring every calculation is not a complete accounting policy.

- Use checked arithmetic for balances, escrow totals, rates, indices, and accrued
  claims. Use the same represented $q$ for source deductions and commissions,
  and validate $Hq<1$ against that represented value.
- Preserve fractional earned credits; whole-unit collection must not turn every
  accrual into whole-unit rounding.
- Carry or otherwise account for small accruals so repeated settlement does not
  systematically erase a child's income or redirect material rewards to a parent.
- Preserve ownership of shared remainders, or explicitly mark them unallocated
  when eligibility changes. A change of participants matters even if the total
  denominator happens to remain unchanged.
- Keep a member's already-earned fractional claim through withdrawals and policy
  changes. Distinguish unclaimed child liabilities from the parent's spendable
  entitlement.
- Reconcile routing, empty-lane, and numerical unallocated amounts against the
  scheduled budget. None becomes an unrecorded reward for a later participant.

Validation must compare the implementation with an exact rational event-driven
reference under varied lane orders, frequent trades, tiny settlements, escrow
changes, and maximum values. It must prove non-overissuance and a useful error
bound under adversarial call schedules. A bound growing without control with the
number of settlement calls is insufficient.

The far-future token-supply/base-unit bound is deferred as described in
[Emission Schedule](Emission-Schedule). This does not defer ordinary overflow
checks or ownership-preserving arithmetic for current operations.

## Earned claims, issued supply, and custody

Scheduled budget, earned-but-uncollected claims, issued credits, and escrowed
principal are distinct quantities:

| Operation | Effect |
| --- | --- |
| Core and marketing accrual | Record net funded entitlement; no mint |
| Collection | Remove earned entitlement and mint the same amount once |
| Ordinary admission | Transfer issued credits to the parent; no burn or reward weight |
| Winning top-seat payment | Burn the authorized price in issued credits |
| Family escrow deposit/withdrawal | Move issued principal into/out of custody; no mint or burn |

Uncollected claims cannot pay a fee or back escrow until collected. A convenience
collect-and-stake operation must realize each claim once before escrowing the
issued tokens. Gifts do not automatically compound. The same issued principal
cannot simultaneously back multiple families or incompatible gauge, clearing,
or auction commitments.

Issued supply reconciles as

```
issued supply = documented bootstrap supply
              + cumulative collected emission claims
              - cumulative burns
```

Escrow is part of issued supply, not an additional claim on emission. Auction or
other authorized burns do not replenish the schedule. Across the emission
pipeline, collected claims, uncollected entitlements, pending accrual, and
explicitly unallocated amounts must reconcile to the scheduled budget without
counting marketed credits twice.

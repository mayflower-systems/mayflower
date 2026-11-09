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
issued credits. This page specifies the accounting that implements the
[referral allocation rules](Referral-Network-Formal-Specification). The first
sections use ideal arithmetic; the finite-precision section defines rounding,
carry ownership, and closure of shared residuals. Neither model permits a
balance change to rewrite an earlier entitlement.

## Lane history and core accrual

Let $E(t)$ be the continuously declining global rate and $e_l(t)$ the
authenticated rate assigned to lane $l$. Let $P_l(t)$ be that lane's total own
eligible asset balance. Its ideal cumulative reward per asset unit is

$$I_l(t)=\int_{t_0}^{t}\mathbf{1}_{P_l(u)>0}\frac{e_l(u)}{P_l(u)}\,du,$$

where the integrand is defined as zero when $P_l(u)=0$.

Allocation to an empty lane is unallocated and unissued, not a backlog for its
next depositor. The external allocator must preserve previous weight intervals;
using today's weight for an entire unprocessed past interval is incorrect.

The tree fixes maximum earning depth $H$ and ancestor commission $q$, with
$Hq<1$. The intended referral configuration uses $H>0$ and $q>0$; the zero
configurations are defined in [Parameters](Referral-Network-Parameters).
Auctioned seats have depth zero. For a member whose own balance $p_{i,l}$ and
strict descendant balance $D_{i,l}$ remain constant between lane checkpoints, an
index increment $\Delta I_l$ produces

$$
\Delta A_{i,l}=(1-d_iq)p_{i,l}\Delta I_l,\qquad
\Delta O_{i,l}=qD_{i,l}\Delta I_l.
$$

The first term is own-asset income; the second is gross referral commission.
Together they are the member's core earnings from that interval. Raw asset units
are combined only within their lane; the resulting credit amounts can be added
across lanes.

Both emission history and allocation history matter. Even while the global rate
declines, a lane with 200 asset units might receive 10 credits in one hour and
90 in the next because its share of that stream increases. In that example, 100
effective core units held only during the first hour earn 5 credits. The same
units held only during the second earn 45. A single average rate over both hours
would lose that history.

Continuous rate variation is integrated into the cumulative lane index. It does
not require a transaction at every instant. Checkpoints at balance, allocation,
escrow, or policy changes preserve the relevant intervals.

## One family index across lanes

For parent $i$, let $s_c$ be each immediate child's issued-credit escrow and
$S_i=\sum_c s_c$. The parent's discretionary marketing fraction is $\mu_i$. Its
cumulative family index $F_i$ measures marketing credits allocated per escrowed
credit, not per BTC or ETH unit.

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
and incoming family allocations do not enter that source. If no child escrows
credits, the parent keeps the otherwise undistributable amount.

A child records its parent's family index at its previous incoming checkpoint:

```
kept_c += s_c * (F_parent - f_c_seen)
f_c_seen = F_parent
```

This family allocation does not update the child's own $F_c$. Family allocations
are terminal, and parents cannot also collect the gross commissions already
assigned to their children. An unchanged child balance can remain lazy while the
parent's index advances; siblings do not all need to be loaded for each
settlement.

## Frequent asset changes

A buy, sale, deposit, withdrawal, or other authenticated eligible-position
change first advances the affected lane to the event time under its old total.
It then settles the affected path from the earning seat toward the member under
the old balances, before updating the member's own balance, every ancestor's
descendant total, and $P_l$.

This preserves earnings before the trade and applies new balances only to future
accrual. Credit escrow does not change automatically with an asset trade, and
trading assets never changes the permanent parent connection.

## The all-source family barrier

**Before changing a parent's marketing fraction, or any child's escrow and hence
the family's total escrow, settle every contributing lane of that parent to the
same effective time using the old family state.**

For an escrow change, then settle the affected child's incoming family
allocation using its old escrow amount. Only after that checkpoint may principal
and the child/family escrow counters change. A marketing change likewise takes
effect immediately after the common checkpoint, for subsequent accrual in every
lane.

The source set must be complete and authenticated. A caller cannot supply only
convenient lanes. Track historical member/lane records that can contain pending
income; a current zero asset balance does not justify forgetting their history.
Since incoming family allocations never fund outgoing marketing, this
parent-source barrier does not require recursively materializing every
ancestor's marketing income.

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
its earned claim. A stake-and-unstake cycle at one timestamp earns no
intervening reward.

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
gross-override sources, then advance the incoming family checkpoint. Iterating
only the claimant's own asset lanes is insufficient: a member with no asset
records can still earn family allocations funded by every lane of its parent.

Collection debits the earned balance and issues the same whole credit amount
once. Fractional earned entitlement remains with that member. A second
collection without new entitlement mints nothing. All required state updates,
custody movements, and mint/burn effects must succeed or fail together.

## Execution cost

A single-lane asset change requires work proportional to affected ancestry. A
credit-escrow or marketing change also processes all contributing lane sources
of the affected parent in the reference implementation. It does not scan every
sibling, but its work is not independent of the number of lanes.

The current accounting representation supports 16 lanes; depth is configured
separately. A family barrier includes every lane ever materialized for that
parent, including withdrawn positions. Shared gauge and lane records must be
committed when residuals are closed, so independent branches can contend for
those records even though no operation loads the whole tree.

The complete barrier must fit the deployment's execution limits. An alternative
that splits it into transactions must preserve an authenticated snapshot and
atomic economic effect: partially processed history cannot activate new stake
against old income. A simple index does not by itself establish that property.

## Finite precision and residual ownership

The following policy makes the ideal equations executable on a finite grid. Let
$Q>0$ be the accounting quantum and define
$\lfloor x\rfloor_Q=Q\lfloor x/Q\rfloor$. Asset balances, credit escrow, gauge
votes, and fees are whole base units. Indices and accrued credits retain
fractional base units on the selected grid. The accounting representation must
hold every intermediate and lifetime total; overflow or an invalid decrease in
an index rejects the action.

### Index division

A shared pool with newly available credits $X$, carried residual $r$, and
integer denominator $n>0$ advances its index by

$$
\Delta J=\left\lfloor\frac{X+r}{n}\right\rfloor_Q,
\qquad r'=X+r-n\Delta J.
$$

Thus $0\le r'<nQ$. This rule applies to the global gauge ($n=V$), a lane
($n=P_l$), and a family ($n=S_i$). Multiplication by integer balances is exact
on the accounting grid, subject to range checks. A zero-vote or empty-asset
interval is recorded as unallocated rather than carried for later arrivals. A
family with no child escrow retains its overrides instead of funding a pool.

### Exact rates and individual carries

Commission and marketing fractions use exact integer numerators over
$D=1{,}000{,}000$: $q=a/D$ and $\mu_i=b_i/D$. The depth check is the exact
integer comparison $Ha<D$. No binary approximation of $q$ is independently used
for its source deduction and ancestor commission.

For a member/lane increment, let $X=p_{i,l}\Delta I_l$ and
$Y=D_{i,l}\Delta I_l$. The own-income numerator is $X(D-d_i a)$ with divisor
$D$. The retained-override numerator is $Ya(D-b_i)$ and the marketed numerator
is $Ya b_i$, each with divisor $D^2$. Use $b_i=0$ for both override calculations
when $S_i=0$.

For each such numerator $z$, divisor $k$, and previous carry $c$:

$$
x=\left\lfloor\frac{z+c}{k}\right\rfloor_Q,
\qquad c'=z+c-kx.
$$

Own-income carries belong to that member's lane position. Retained-override
carries belong to the parent individually. They survive asset withdrawals and
family-policy changes. Marketing-rate carries belong to the shared family
interval; they cannot be assigned to newcomers. Separate carries ensure that a
fraction reserved for children is never first credited as the parent's income.

### Closing an eligibility interval

After settling old terms, an actual change closes the following shared
residuals:

| Change                              | Residuals moved to explicit unallocated accounting                                                          |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Gauge weight                        | Global gauge division residual                                                                              |
| Asset balance                       | Global gauge residual and the affected lane's division residual                                             |
| Family escrow or marketing fraction | Global gauge residual, every parent source lane's residual, family-index residual, and marketing-rate carry |

A closed residual is not awarded to later participants or returned as extra
parent income. The marketing-rate carry is recorded in its exact numerator
units; its credit value is that counter divided by $D^2$. A change in
participants matters even if another action later restores the old denominator.
No-op settlement closes nothing. Already-attributed individual fractions remain
owned.

Closing a source lane's shared residual also affects other members of that lane:
no one had yet received that residual as an individual entitlement. This is an
explicit precision loss, not confiscation of accrued personal claims.

For one actual asset change the closed amount is at most $Q(V+P_l)$. For one
family change it is less than

$$Q\left(V+\sum_{l\in\mathcal L_i}P_l+S_i+1\right),$$

where $\mathcal L_i$ is the parent's complete historical source set. A gauge
change closes at most $QV$. A zero denominator has zero residual; the gauge and
asset bounds are strict when their denominator sums are positive. These are
per-change bounds, not a lifetime cap; cumulative unallocated amounts remain
recorded. Precision and supported totals must make them acceptable for the
deployment. Repeated settlement with unchanged eligibility carries fractions
forward and does not incur these closures.

Collection alone rounds down to whole credit base units and retains the member's
remaining fraction. Neither collection nor a change of terms permits double
issuance against parent income and child liabilities.

The far-future token-supply/base-unit bound is deferred as described in
[Emission Schedule](Emission-Schedule). That does not defer checked arithmetic,
range analysis, or numerical validation for current operations.

## Earned claims, issued supply, and custody

Scheduled budget, earned-but-uncollected claims, issued credits, and escrowed
principal are distinct quantities:

| Operation                        | Effect                                                          |
| -------------------------------- | --------------------------------------------------------------- |
| Core and marketing accrual       | Record net funded entitlement; no mint                          |
| Collection                       | Remove earned entitlement and mint the same amount once         |
| Ordinary admission               | Transfer issued credits to the parent; no burn or reward weight |
| Winning top-seat payment         | Burn the authorized price in issued credits                     |
| Family escrow deposit/withdrawal | Move issued principal into/out of custody; no mint or burn      |

Uncollected claims cannot pay a fee or back escrow until collected. A
convenience collect-and-stake operation must realize each claim once before
escrowing the issued tokens. Family allocations do not automatically compound.
The same issued principal cannot simultaneously back multiple families or
incompatible gauge, clearing, or auction commitments.

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

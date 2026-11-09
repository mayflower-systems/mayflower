<!--
title: Referral Network Formal Specification
summary:
  Equations and action contracts for per-lane allocation, credit-escrow mutualization,
  conservation, and permanent referral geometry.
tags:
  - referral
  - math
-->

This page specifies the [Referral Network](Referral-Network): permanent
membership, credit allocation, admission, and custody. Its rate equations use
ideal arithmetic. The companion [Emission Accounting](Emission-Accounting)
specifies chronological settlement, cumulative indices, and finite-precision
rules. [Rationale](Referral-Network-Rationale) explains the incentives and their
limits; it does not add exceptions to the rules.

## Geometry and notation

There is one logical non-earning protocol root. Its scarce auctioned seats have
**earning depth 0**; the protocol root receives no override. Every ordinary node
has one permanent parent, and a node at maximum depth $`H`$ cannot accept
children.

```math
H\in\mathbb{Z}_{\ge0},\qquad 0\le q\le1,\qquad Hq\lt 1,\qquad 0\le d_i\le H.
```

$`H`$ and $`q`$ are fixed network terms. $`d_i`$ counts earning ancestors, so a
maximum-depth path contains $`H+1`$ earning members. The intended referral
configuration uses $`H\gt 0`$ and $`q\gt 0`$. The general allocation rule also permits
$`H=0`$ (only auctioned seats) and $`q=0`$ (no ancestor commissions).

A member cannot change parent, insert an intermediary into an existing edge,
sever its ancestry, or create a cycle. Withdrawing all balances does not delete
membership or descendants. One owner may control multiple members.

| Symbol         | Meaning                                                        |
| -------------- | -------------------------------------------------------------- |
| $`E(t)`$       | Global credit emission rate, decreasing continuously over time |
| $`e_\ell`$     | Credit emission rate allocated to asset lane $`\ell`$          |
| $`p_{i,\ell}`$ | Member $`i`$'s own eligible asset balance in that lane         |
| $`D_{i,\ell}`$ | Sum of eligible balances of all strict descendants of $`i`$    |
| $`P_\ell`$     | Sum of all members' own eligible balances in the lane          |
| $`s_i`$        | Issued credits escrowed by $`i`$ with its immediate parent     |
| $`S_i`$        | Sum of credit escrow over $`i`$'s immediate children           |
| $`\mu_i`$      | Member $`i`$'s discretionary marketing fraction, from 0 to 1   |

$`D`$ excludes the member's own balance. $`S`$ excludes descendant escrow beyond
the immediate children. Credits in family escrow do not enter any asset
denominator.

## Core allocation

The following rate equations apply at time $`t`$; time arguments are omitted for
readability. Interval entitlements integrate these rates over their actual
history, including changes to emission and balances.

For $`P_\ell\gt 0`$, member $`i`$'s own-asset and override income rates are

```math
A_{i,\ell}=\frac{e_\ell}{P_\ell}(1-d_iq)p_{i,\ell},
\qquad
O_{i,\ell}=\frac{e_\ell}{P_\ell}qD_{i,\ell}.
```

Its total core income and gross override income are

```math
C_i=\sum_\ell(A_{i,\ell}+O_{i,\ell}),\qquad O_i=\sum_\ell O_{i,\ell}.
```

Each source funds one $`q`$ commission for each of its $`d_i`$ earning ancestors
and retains the complement. Therefore

```math
\sum_i\bigl[(1-d_iq)p_{i,\ell}+qD_{i,\ell}\bigr]=P_\ell.
```

Core income sums to each nonempty lane's assigned emission. An ancestor's cut is
calculated from the source's original attribution, not compounded from what
another ancestor leaves behind. A member can earn overrides without own assets.

In ordinary active operation the allocator supplies
$`\sum_\ell e_\ell(t)=E(t)`$. The interval budget is $`\int E(t)\,dt`$.
Unavailable routing, empty-lane allocation, and numerical residuals must be
recorded as unallocated, unissued budget. An empty lane has no earned
allocation, and later deposits receive no backpay. Asset values are compared
within lanes; the external allocator determines their relative credit budgets.

## Discretionary mutualization

The parent's outgoing marketing rate is

```math
M_i=\begin{cases}
\mu_i O_i,&S_i\gt 0,\\
0,&S_i=0.
\end{cases}
```

For an immediate child $`c`$, when $`S_i\gt 0`$:

```math
G_{i\to c}=M_i\frac{s_c}{S_i}.
```

The member's final income rate is

```math
R_i=C_i-M_i+G_{parent(i)\to i},
```

with incoming family allocation zero for an auctioned seat. Every family
allocation is a deduction from one member and an addition to another, so
$`\sum_iR_i=\sum_iC_i`$.

Only gross overrides fund $`M_i`$. Own-asset earnings, admission fees, escrow
principal, auction payments, and incoming family allocations are excluded.
Incoming family allocations are kept in full and never shared again. A
zero-escrow child receives no family family allocation; if $`S_i=0`$, the parent
keeps the override income.

A parent can change $`\mu_i`$ prospectively at any time. The previous terms must
be settled across all contributing lanes before the change. Credit escrow is
withdrawable after settling earlier entitlement; no permanent income ownership
or mandatory lock is created.

## Admission and custody

Ordinary admission transfers the parent's current credit fee to its owner and
creates the immutable edge. The parent's nonnegative fee increment applies once
to a successful join, saturating at the representable maximum. Zero fee and zero
increment are valid. The parent may edit future prices, and a payer can enforce
a maximum accepted fee.

Top-level admission requires a scarce, authenticated auction allocation. The
winning payment in already-issued credits is burned, rather than paid to a
member. The first paid auction requires a separately documented existing or
genesis credit inventory; this model does not invent a free first seat or mint
its own auction payment.

Family escrow remains the child's principal and cannot simultaneously back
another family stake or be spent. Accrued rewards become issued credits when
collected once; uncollected claims do not count as escrow. A collect-and-stake
operation performs those steps in that order.

## State invariants and action contracts

The authoritative aggregates are

```math
P_l=\sum_i p_{i,l},\qquad
D_{i,l}=\sum_{c\text{ child of }i}(p_{c,l}+D_{c,l}),\qquad
S_i=\sum_{c\text{ child of }i}s_c.
```

Each asset position enters $`P_l`$ once. Descendant caches do not create
additional eligible assets. Custody must prevent two records from claiming the
same underlying position. A member's historical set of lane records remains
complete even when a balance becomes zero.

New members start with zero assets, family escrow, child escrow, marketing
fraction, and accrued entitlement. An ordinary child's incoming family
checkpoint starts at its parent's current stored index; the first positive
escrow still requires full settlement under the old family terms. Missing lane
positions begin at the lane's current index, with no claim on earlier emission.

An admission offer contains two unsigned whole credit-base-unit amounts,
`current_fee` and `increment`, each at most $`2^{64}-1`$. The price after a
successful join is

```math
\mathrm{next\_fee}=\min(\mathrm{current\_fee}
+\mathrm{increment},\;2^{64}-1).
```

This saturation is a quote rule. Other balances, totals, and accrued claims use
checked arithmetic; they must not silently saturate.

| Action                      | Required transition                                                                                                                                                                    |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Activate an auctioned seat  | Authenticate the unique award, consume it once, burn its final issued-credit price, and create a depth-zero member. Losing or refundable bids are not winning payments.                |
| Join under a parent         | Authenticate the existing parent and new identity, check remaining depth and the payer's maximum fee, transfer the current fee, create the permanent edge, and advance the quote once. |
| Change an offer             | The parent owner changes the fee or increment for future joins; existing edges and earnings are unchanged.                                                                             |
| Change an asset balance     | Settle the affected lane and complete ancestor path at the old balances, then update the member's own balance, the lane total, and each ancestor's descendant total.                   |
| Change a marketing fraction | The owner settles every contributing lane under the old fraction and escrow total before installing the new fraction.                                                                  |
| Change child escrow         | Settle every parent source and the child's incoming family allocation under its old escrow, then change that principal and the parent's child-escrow total together.                   |
| Change gauge allocation     | Settle the old routing interval before applying new lane weights.                                                                                                                      |
| Collect credits             | Remove accrued net whole credit base units and issue that amount once; retain the member's fractional claim.                                                                           |

All state changes and required custody movements are atomic. Rejected actions
transfer nothing and change no price or entitlement. Authorization must bind
each state record to its protocol, lane, member, or auction award; a caller
cannot supply a different account with convenient cached totals. The emission
snapshot must be authenticated, chronological, and consistent at a given
timestamp.

Top-level seats cannot escrow into the non-earning root. A child can partially
or fully withdraw family principal after settlement, without a mandatory lock,
penalty, or forfeiture. The same issued principal cannot simultaneously back
family escrow, gauge pledges, clearing commitments, or auction bids.

Collection may issue only the claims already materialized. To collect all
entitlement through a specified time, first settle the member's core sources,
its immediate parent's gross-override sources, and its incoming family index. A
child with no asset records can still earn from every source lane of its parent.
[Emission Accounting](Emission-Accounting) specifies those checkpoints and the
ownership of fractional residuals.

## Guarantees and implementation boundary

The ideal model conserves allocated emission. It also gives two limited
splitting properties: moving fixed assets within a connected owned region under
unchanged outside ancestry does not improve its combined core attribution, and
splitting the same credit escrow among aliases under one parent does not improve
their combined family share.

These are not universal Sybil or coalition-resistance claims. See
[Rationale](Referral-Network-Rationale) for their scope. Before deployment,
implementations must validate numerical bounds, remainder ownership, exclusive
custody, and complete historical settlement. The formulas alone do not establish
those implementation guarantees.

<!--
title: Referral Network Formal Specification
summary:
  The target equations for per-lane core allocation, credit-escrow mutualization,
  conservation, and permanent referral geometry.
tags:
  - referral
  - math
-->

This page summarizes the [Referral Network](Referral-Network). The
[implementation specification](https://github.com/mayflower-systems/mayflower/blob/a1f87bfa8469630444626ddbc11536512c78bb16/referral/rust/mayf-referral/SPEC.md)
is the authoritative handoff for state, actions, custody, and acceptance tests.
The kernel implements override-only marketing with credit escrow and complete
source settlement before prospective changes.

All equations here use ideal arithmetic. [Emission Accounting](Emission-Accounting)
is the canonical wiki page for chronological settlement, multi-lane checkpoints,
collection, and finite-precision requirements.

## Geometry and notation

There is one logical non-earning protocol root. Its scarce auctioned seats have
**earning depth 0**; the protocol root receives no override. Every ordinary node
has one permanent parent, and a node at maximum depth $H$ cannot accept children.

$$H\in\mathbb{Z}_{>0},\qquad q>0,\qquad Hq<1,\qquad 0\le d_i\le H.$$

$H$ and $q$ are fixed network terms. $d_i$ counts earning ancestors.

| Symbol | Meaning |
| --- | --- |
| $E(t)$ | Global credit emission rate, decreasing continuously over time |
| $e_\ell$ | Credit emission rate allocated to asset lane $\ell$ |
| $p_{i,\ell}$ | Member $i$'s own eligible asset balance in that lane |
| $D_{i,\ell}$ | Sum of eligible balances of all strict descendants of $i$ |
| $P_\ell$ | Sum of all members' own eligible balances in the lane |
| $s_i$ | Issued credits escrowed by $i$ with its immediate parent |
| $S_i$ | Sum of credit escrow over $i$'s immediate children |
| $\mu_i$ | Member $i$'s discretionary marketing fraction, from 0 to 1 |

$D$ excludes the member's own balance. $S$ excludes descendant escrow beyond the
immediate children. Credits in family escrow do not enter any asset denominator.

## Core allocation

The following rate equations apply at time $t$; time arguments are omitted for
readability. Interval entitlements integrate these rates over their actual
history, including changes to emission and balances.

For $P_\ell>0$, member $i$'s own-asset and override income rates are

$$
A_{i,\ell}=\frac{e_\ell}{P_\ell}(1-d_iq)p_{i,\ell},
\qquad
O_{i,\ell}=\frac{e_\ell}{P_\ell}qD_{i,\ell}.
$$

Its total core income and gross override income are

$$C_i=\sum_\ell(A_{i,\ell}+O_{i,\ell}),\qquad O_i=\sum_\ell O_{i,\ell}.$$

Each source funds one $q$ commission for each of its $d_i$ earning ancestors and
retains the complement. Therefore

$$\sum_i\bigl[(1-d_iq)p_{i,\ell}+qD_{i,\ell}\bigr]=P_\ell.$$

Core income sums to each nonempty lane's assigned emission. An ancestor's cut is
calculated from the source's original attribution, not compounded from what
another ancestor leaves behind. A member can earn overrides without own assets.

In ordinary active operation the allocator supplies
$\sum_\ell e_\ell(t)=E(t)$. The interval budget is $\int E(t)\,dt$.
Unavailable routing, empty-lane allocation, and numerical residuals must be
recorded as unallocated, unissued budget. An empty lane has no earned allocation,
and later deposits receive no backpay. Asset values are compared within lanes;
the external allocator determines their relative credit budgets.

## Discretionary mutualization

The parent's outgoing marketing rate is

$$
M_i=\begin{cases}
\mu_i O_i,&S_i>0,\\
0,&S_i=0.
\end{cases}
$$

For an immediate child $c$, when $S_i>0$:

$$G_{i\to c}=M_i\frac{s_c}{S_i}.$$

The member's final income rate is

$$R_i=C_i-M_i+G_{parent(i)\to i},$$

with incoming gift zero for an auctioned seat. Every gift is a deduction from
one member and an addition to another, so $\sum_iR_i=\sum_iC_i$.

Only gross overrides fund $M_i$. Own-asset earnings, admission fees, escrow
principal, auction payments, and incoming gifts are excluded. Incoming gifts
are kept in full and never shared again. A zero-escrow child receives no family
gift; if $S_i=0$, the parent keeps the override income.

A parent can change $\mu_i$ prospectively at any time. The previous terms must
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

## Guarantees and implementation boundary

The ideal model conserves allocated emission. It also gives two limited splitting
properties: moving fixed assets within a connected owned region under unchanged
outside ancestry does not improve its combined core attribution, and splitting
the same credit escrow among aliases under one parent does not improve their
combined family share.

These are not universal Sybil or coalition-resistance claims. See
[Rationale](Referral-Network-Rationale) for their scope. Before deployment,
implementations must validate numerical bounds, remainder ownership, exclusive
custody, and complete historical settlement. The formulas alone do not establish
those implementation guarantees.

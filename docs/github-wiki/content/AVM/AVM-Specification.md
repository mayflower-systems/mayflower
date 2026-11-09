<!--
title: AVM Specification
summary:
  State, pricing, backing, settlement, and numerical rules for the AVM.
tags:
  - avm
  - math
-->

This page specifies the [Amplifying Value Machine](Amplifying-Value-Machine).
The equations describe exact arithmetic unless the numerical rules explicitly
say otherwise. [AVM Rationale](AVM-Rationale) explains the design choices;
[AVM Evolution](AVM-Evolution) gives the state transitions.

## State and units

Each market fixes a smooth, strictly increasing characteristic function $`g(x)`$
with $`g(0)=0`$ and a ramp multiplier $`k\gt 1`$. The function and multiplier
remain fixed throughout the market's lifetime. The characteristic is evaluated
on its supported nonnegative domain; out-of-domain inputs and unrepresentable
arithmetic fail.

| Symbol          | Meaning                                               |
| --------------- | ----------------------------------------------------- |
| $`F\ge0`$       | Floor price, in reserve units per AVA                 |
| $`E\ge0`$       | Ramp end, on the progress axis                        |
| $`W\in[0,E]`$   | Ramp width                                            |
| $`A=E-W`$       | Ramp start                                            |
| $`b`$           | Signed vertical offset of the main schedule           |
| $`P\ge E`$      | Pricing progress, in AVA units                        |
| $`T\ge0`$       | Actual circulating AVA supply                         |
| $`J=T-P+A\ge0`$ | Physical floor-tail quantity                          |
| $`H\ge0`$       | Liquid reserve held by the market                     |
| $`D\ge0`$       | Outstanding reserve advances secured by AVA positions |
| $`B`$           | Required backing for the current circulation          |

Progress and circulation are separate quantities. Ordinary issuance advances
both; floor issuance and special floor redemption change only circulation.
Consequently, price must be evaluated at $`P`$, not at $`T`$.

## Price schedule

Define the main and ramp functions:

```math
M(x)=g(x)+b,
\qquad
R(x)=M(E)+k\bigl(g(x)-g(E)\bigr).
```

The schedule is

```math
y(x)=
\begin{cases}
F,&0\le x\le A,\ x\lt E,\\
R(x),&A\lt x\lt E,\\
M(x),&x\ge E.
\end{cases}
```

At $`E`$, including when $`A=E`$, main takes precedence. At $`A\lt E`$, the point
value is $`F`$ even if an upward floor-join gap is permitted. The ramp meets
main at $`E`$ by construction. In the continuous reference model, $`R(A)=F`$; if
$`W=0`$, this condition means $`M(E)=F`$. Numerical acceptance may permit a
bounded upward floor-join gap, as specified below. A downward gap is not
permitted.

The spot price is $`y(P)=M(P)`$. For $`A\lt x\lt E`$,

```math
M(x)-R(x)=(k-1)\bigl(g(E)-g(x)\bigr)\gt 0.
```

This is the **ramp constraint**: the ramp occupies less area than extending main
backward over the same interval. Floor raises can use this difference to fund a
higher floor.

## Launch

A fresh market starts with

```math
P=T=E=W=0,\qquad F=F_0\ge0,\qquad b=F_0.
```

Its initial schedule is $`M(x)=g(x)+F_0`$ and required backing is zero. An
initial donation may add surplus reserve. A nonempty restored market must also
satisfy the geometry, backing, custody, and collateral conditions in this
specification.

## Required backing and reserve coverage

The physical floor tail has $`J`$ AVA units. The remaining $`P-A`$ units are
backed by the ramp and main portions of the schedule:

```math
B=FJ+\int_A^P y(x)\,dx.
```

Equivalently,

```math
B=\int_0^P y(x)\,dx+(T-P)F
 =FT+\int_A^P\bigl(y(x)-F\bigr)\,dx.
```

When $`T=P`$, this reduces to the area from zero to circulation. In general,
circulation cannot replace progress as the integration limit. The backing
condition is

```math
H+D\ge B.
```

Equality describes an exactly funded market without surplus. Donations and
conservative rounding can leave surplus. Advances move backing from liquid
reserve into collateralized positions; $`H+D`$ is an accounting total, not an
amount immediately available for transfer. Every payout must separately be
covered by liquid reserve after any simultaneous repayment or purchase.

For a position with $`n`$ AVA and outstanding advance $`d`$, the collateral rule
is $`d\le Fn`$. Collateral cannot also be transferred or redeemed free of its
advance. [Cash Advance Mechanics](Cash-Advance-Mechanics) describes combined
settlement.

## Issuance and redemption

For ordinary issuance of $`n\gt 0`$:

```math
C=\int_P^{P+n}y(x)\,dx,\qquad P'=P+n,\qquad T'=T+n.
```

The buyer supplies $`C`$ of backing plus the issuance fee. Geometry is
unchanged. Since $`P\ge E`$, ordinary purchases begin on main.

For ordinary redemption of $`0\lt n\le T`$, set $`Q=\max(P-n,0)`$. Gross proceeds
are quoted on the **old** schedule:

```math
V=\int_Q^P y(x)\,dx+F\max(n-P,0),
\qquad P'=Q,\qquad T'=T-n.
```

If the sale reaches below the old ramp end, the schedule contracts after the
quote. The exact contraction rules are in [AVM Evolution](AVM-Evolution).
Redemption fees and any required advance repayment reduce the holder's net
payout. Numerical contraction may also require a conservative backing deduction.

Authorized [floor issuance](Call-Options) adds $`n`$ AVA and $`Fn`$ of backing
while leaving $`P`$ and geometry unchanged. Special floor redemption removes
$`0\lt n\le J`$ AVA for gross proceeds $`Fn`$, also leaving $`P`$ and geometry
unchanged. Both operations change $`T`$ and $`J`$ by the same amount. Special
floor redemption is bounded by the available floor tail; it is not an unlimited
liquidity source.

## Floor raises

A proposal chooses $`F'\gt F`$, $`E'`$, and $`W'`$, with $`A'=E'-W'`$. It must
satisfy

```math
0\le A'\le E'\le P,\qquad A'\ge A,\qquad E'\ge E.
```

The main offset $`b`$, characteristic $`g`$, multiplier $`k`$, progress $`P`$,
and circulation $`T`$ are unchanged. These conditions preserve the live main
price. Both joins move forward or stay fixed; the ramp width may grow or shrink.

The proposal must pass the floor-join check and the chosen backing-change
policy. An area-preserving raise has $`B'=B`$ in exact arithmetic. A
[surplus-funded raise](Floor-Investment) permits $`B'\ge B`$, funded by existing
reserve surplus. Both require $`H+D\ge B'`$ and change no reserve balance by
themselves. [AVM Floor Raising](AVM-Floor-Raising) defines the full checks.

## Numerical and settlement rules

Calculations produce lower and upper bounds enclosing exact values. Issuance
charges the upper reserve bound; redemption pays the lower bound. Rounding an
updated main offset upward during contraction may increase retained backing;
that increase is reserved from redemption proceeds. An operation fails when its
bounds cannot certify the required inequalities.

For a floor raise, let $`G`$ be the new ramp's value at $`A'`$ minus $`F'`$; for
a zero-width ramp use $`M(E')-F'`$. With configured allowances $`\epsilon_p`$ in
price units and $`\epsilon_B`$ in reserve units:

- The entire enclosure for $`G`$ must lie in $`[0,\epsilon_p]`$.
- An ordinary raise requires the entire enclosure for $`B'-B`$ to lie in
  $`[-\epsilon_B,0]`$.
- A surplus-funded raise requires it to lie in $`[0,\epsilon_B]`$.
- The upper bound for $`B'`$ must be covered by $`H+D`$.

These are directional acceptance intervals, not symmetric error margins. Setting
both allowances to zero requests exact joins and exact backing preservation.

Fees are charged in the reserve asset. With integer fee rate $`r`$ and
denominator $`10^6`$, a fee on amount $`v`$ is $`\lceil vr/10^6\rceil_\delta`$,
where $`\lceil\cdot\rceil_\delta`$ rounds upward to the chosen arithmetic grid.
The protocol-fee allocation is rounded upward in the same way; the market-owner
fee allocation receives the remainder. The protocol rate applies to the
collected operation fee, not directly to the transaction's full reserve amount.
Issuance, redemption, newly drawn advances, and issuance or redemption at the
floor use their respective configured fees. Advance repayment has no fee, and
configured rates may be zero. Token-unit normalization and final transfer
rounding must preserve these conservative charge and payout bounds.

In the intended Mayflower integration, each fee-bearing AVM action transfers its
protocol allocation into [revenue clearing](Revenue-Clearing) in the same
transaction. The destination is a pot for the market's base reserve asset in an
epoch accepting funding. Token transfer and recognition of funding must succeed
together with the AVM transition; a failed required funding step fails the
transaction. There is no deferred transfer through an operator's fee wallet or
conversion into another asset. This routing belongs to the integrating AVM
program, while the arithmetic specifies the fee amounts.

An integrating application must authenticate market and position state, bind the
same characteristic parameters, enforce collateral custody and any option
authorization, and atomically settle token supply, reserve transfers, fees, and
new state. The arithmetic alone does not establish those external facts.

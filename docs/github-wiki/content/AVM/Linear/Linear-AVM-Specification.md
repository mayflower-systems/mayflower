<!--
title: Linear AVM Specification
summary:
  Exact pricing, backing, and state formulas for the linear AVM characteristic.
tags:
  - linear-avm
  - math
-->

The linear AVM uses $g(x)=mx$ in the [general specification](AVM-Specification),
with fixed main slope $m>0$ and ramp multiplier $k>1$. Write $a=km$ for the ramp
slope, so $a>m$. The formulas here assume exact continuous joins; numerical
acceptance follows the general specification.

## State and schedule

| Symbol       | Meaning                                    |
| ------------ | ------------------------------------------ |
| $F$          | Floor price                                |
| $b$          | Main intercept                             |
| $E$          | Ramp end                                   |
| $W$          | Ramp width                                 |
| $A=E-W$      | Ramp start                                 |
| $P$          | Pricing progress, with $P\ge E$            |
| $T$          | Circulating AVA quantity                   |
| $J=T-P+A$    | Physical floor-tail quantity, with $J\ge0$ |
| $c=b-(a-m)E$ | Derived ramp intercept                     |

The schedule consists of

$$
\text{floor}:F,\qquad R(x)=ax+c,\qquad M(x)=mx+b.
$$

Floor applies for $x\le A$ below $E$, ramp for $A<x<E$, and main for $x\ge E$.
At a zero-width ramp, main takes precedence at the common endpoint. Exact
continuity at the floor requires

$$
F=aA+c.
$$

The joins can equivalently be recovered from the intercepts:

$$
A=\frac{F-c}{a},\qquad E=\frac{b-c}{a-m}.
$$

Those equalities assume zero floor-join gap. With a permitted numerical upward
gap, the stored start $A=E-W$ is the boundary and the equality at the floor is
replaced by the explicit acceptance check.

Launch has $P=T=E=A=0$ and $b=F_0$. Subsequent floor raises preserve $b$;
ordinary redemption into the ramp or floor can change it. Floor issuance
changes neither intercept nor either boundary.

## Required backing

Let $y_E=mE+b$ and $y_P=mP+b$. The floor, ramp, and main contributions are

$$
B=FJ+\frac{F+y_E}{2}(E-A)+\frac{y_E+y_P}{2}(P-E).
$$

An equivalent expression convenient for floor-raise calculations is

$$
B=\frac{mP^2}{2}+bP+(T-P)F
  +\frac{aA^2-(a-m)E^2}{2}.
$$

These formulas include circulation that differs from progress. Held reserve
$H$ and outstanding collateralized advances $D$ must cover $B$; liquid payouts
also require sufficient $H$ after combined settlement. The reserve need not
equal $B$ when surplus exists.

### Example

Choose $m=8$, $k=25/16$, so $a=25/2$, with $b=1$. After issuing six AVA from
launch, $P=T=6$ and required backing is

$$
B=\frac{8\cdot6^2}{2}+1\cdot6=150.
$$

A floor raise to $F=16$, $A=3$, $E=5$ preserves that backing. The ramp intercept
is $c=1-(9/2)5=-43/2$. Its price runs from 16 to 41; main ends at 49:

$$
B=16\cdot3+\frac{16+41}{2}\cdot2
 +\frac{41+49}{2}\cdot1=48+57+45=150.
$$

## Ordinary issuance

An issuance of $n>0$ begins on main and costs

$$
C=n\left(mP+b+\frac{mn}{2}\right).
$$

It sets $P'=P+n$, $T'=T+n$, without changing geometry. The gross average fill
price is $mP+b+mn/2$, and the new spot price is $m(P+n)+b$. Fees are additional.

## Ordinary redemption

For $0<n\le T$, let $Q=\max(P-n,0)$. Split $[Q,P]$ at $A$ and $E$ and sum
the areas on the old schedule. For a linear segment $sx+t$ over $[u,v]$:

$$
\int_u^v(sx+t)\,dx=(v-u)\frac{(su+t)+(sv+t)}2.
$$

Use $s=0,t=F$ for floor, $s=a,t=c$ for ramp, and $s=m,t=b$ for main. Add
$F\max(n-P,0)$ for any quantity beyond progress. This gives gross proceeds;
fees and numerical backing deductions affect the actual payout.

Then set $P'=Q$ and $T'=T-n$ and apply contraction:

| Remaining progress | New boundaries | New main intercept |
| ------------------ | -------------- | ------------------ |
| $Q\ge E$           | Unchanged      | $b'=b$             |
| $A<Q<E$            | $A'=A$, $E'=Q$ | $b'=b+(a-m)(Q-E)$  |
| $Q<E$ and $Q\le A$ | $A'=E'=Q$      | $b'=F-mQ$          |

Inside the ramp, its intercept $c$ stays fixed. On floor collapse, the derived
ramp intercept becomes $c'=F-aQ$, but the ramp has zero width. The next purchase
starts on main. See [Linear AVM Evolution](Linear-AVM-Evolution).

## Floor operations

Floor issuance of $n$ AVA adds $Fn$ backing and changes $T$ to $T+n$.
Special floor redemption of $n\le J$ removes $Fn$ backing and changes $T$ to
$T-n$. Both preserve $P,F,A,E,b,c$.

A floor raise increases $F$, preserves $P,T,b$, and supplies $A',E'$ with
$A'\ge A$, $E'\ge E$, and $A'\le E'\le P$. The new ramp intercept is
$c'=b-(a-m)E'$. Its exact floor join requires $F'=aA'+c'$.

[Linear AVM Floor Raising](Linear-AVM-Floor-Raising) solves the constant-backing
case. [Linear AVM Floor Investment](Linear-AVM-Floor-Investment) calculates the
surplus-funded increase.

## Price sensitivity and exposure

The slopes are sensitivities to progress, in reserve-per-AVA per AVA. They are
not statistical measures of volatility. Ordinary buys use slope $m$; a sale can
cross the steeper ramp before reaching floor-priced units.

The backing decomposes as $B=FT+(B-FT)$. The first term is gross floor backing;
the second is value above the floor. Neither the current spot quote nor a rising
floor establishes an individual holder's realized return. Cost basis, trade
size, fees, reserve denomination, and advances all affect that outcome.

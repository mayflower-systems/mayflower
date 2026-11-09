<!--
title: AVM Floor Raising
summary:
  Geometry, backing conservation, and numerical acceptance of a floor raise.
tags:
  - avm
  - floor
-->

A floor raise increases the minimum reserve-denominated redemption value of
AVA while preserving the live price. The
[AVM specification](AVM-Specification) defines the schedule and backing; this
page specifies the proposal and acceptance conditions. A surplus-funded variant
is described in [Floor Investment](Floor-Investment).

## Proposal and fixed quantities

Let the old floor, ramp start, ramp end, progress, and circulation be
$F,A,E,P,T$. A proposal supplies $F'>F$, a new ramp end $E'$, and a new width
$W'$, giving $A'=E'-W'$. It must satisfy

$$
0\le A'\le E'\le P,\qquad A'\ge A,\qquad E'\ge E.
$$

The characteristic $g$, ramp multiplier $k$, main offset $b$, progress $P$,
and circulation $T$ are unchanged. Main remains $M(x)=g(x)+b$, so the live
price stays $M(P)$. Both joins move forward or remain fixed, but the ramp's
width is allowed to decrease.

The new ramp is anchored to unchanged main:

$$
R'(x)=M(E')+k\bigl(g(x)-g(E')\bigr).
$$

For exact continuity, $R'(A')=F'$. If the ramp has zero width, the condition is
$M(E')=F'$.

## Required backing

The old and proposed backing are

$$
B=F(T-P+A)+\int_A^P y(x)\,dx,
$$

$$
B'=F'(T-P+A')+\int_{A'}^P y'(x)\,dx.
$$

An exact area-preserving raise requires $B'=B$. Because main is unchanged beyond
$E'$, the backing difference can also be written as

$$
\Delta B=\int_0^{E'}\bigl(y'(x)-y(x)\bigr)\,dx+(T-P)(F'-F).
$$

The second term is essential after floor issuance or special floor redemption.
Integrating only from zero to progress is insufficient when circulation differs
from progress.

Both held reserve $H$ and outstanding collateralized advances $D$ stay fixed.
The proposed backing must remain covered: $H+D\ge B'$. Raising the floor does
not create reserve assets.

## Why the geometry can fund a raise

Within a nonempty ramp,

$$
M(x)-R(x)=(k-1)\bigl(g(E)-g(x)\bigr)>0.
$$

Moving the ramp end forward replaces part of the old main tail with a lower
ramp. This can offset the extra backing required by the higher floor. A valid
proposal balances the complete backing expression, including the physical floor
tail. The operation redistributes existing backing from price-sensitive value
into floor value.

The available raise depends on the current geometry, characteristic, circulation,
and progress. Every proposal must satisfy the complete geometry and backing
checks above. The condition $P>E$ alone does not establish that a proposed floor
is feasible. The same checks apply to zero-circulation states and to numerical
proposals with a permitted floor-join gap.

## Numerical acceptance

Let $G=R'(A')-F'$ be the floor-join gap; use $M(E')-F'$ when $W'=0$. An ordinary
raise supplies two nonnegative allowances:

| Allowance    | Units           | Required interval for the entire calculated enclosure |
| ------------ | --------------- | ----------------------------------------------------- |
| $\epsilon_p$ | Reserve per AVA | $G\in[0,\epsilon_p]$                                  |
| $\epsilon_B$ | Reserve         | $\Delta B\in[-\epsilon_B,0]$                          |

An upward floor-join step may be allowed; a downward step is rejected. The
backing allowance permits only a bounded decrease, never an increase. The upper
bound for $B'$ must also be covered by $H+D$.

The two allowances are independent. Neither permits the main offset to change.
They are absolute quantities in the stated units, not percentages. Exact
preservation sets them both to zero; a coarse numeric grid may require a small
conservative allowance to admit a candidate.

## Finding and verifying a proposal

A proposer can solve for geometry off-chain and submit the resulting floor
increment, ramp end, and width. Acceptance checks the supplied candidate against
all constraints; it does not rely on the proposer's claimed precision or reserve
calculation. A candidate that cannot be certified within the allowances is
rejected.

[Linear AVM Floor Raising](Linear-AVM-Floor-Raising) gives closed-form equations
and a complete example for the linear characteristic. Other characteristics
require their own numerical search, while retaining these same acceptance rules.

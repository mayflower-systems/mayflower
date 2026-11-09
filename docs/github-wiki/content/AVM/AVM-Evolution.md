<!--
title: AVM Evolution
summary:
  State transitions for trading, contraction, floor operations, and reserve advances.
tags:
  - avm
  - pricing
-->

This page gives the exact state transitions of the
[AVM specification](AVM-Specification). Use $`F`$ for floor price, $`A`$ and
$`E`$ for ramp start and end, $`W=E-A`$, $`P`$ for pricing progress, $`T`$ for
circulation, and $`b`$ for the main offset. The immutable characteristic is
$`g`$, the ramp multiplier is $`k\gt 1`$, and $`J=T-P+A`$ is the physical floor
tail.

## Ordinary purchase

A purchase of $`n\gt 0`$ costs $`\int_P^{P+n}y(x)\,dx`$ before fees and sets

```math
P'=P+n,\qquad T'=T+n.
```

All schedule parameters stay fixed. Because a valid state has $`P\ge E`$, the
purchase follows main. Required backing and held reserve increase by the gross
purchase cost. The floor tail $`J`$ is unchanged.

## Ordinary redemption

For $`0\lt n\le T`$, define $`Q=\max(P-n,0)`$. Quote the entire sale on the old
schedule, with any quantity beyond $`P`$ paid at $`F`$:

```math
V=\int_Q^P y(x)\,dx+F\max(n-P,0).
```

Then set

```math
P'=Q,\quad T'=T-n,\quad E'=\min(E,Q),\quad A'=\min(A,Q),\quad W'=E'-A'.
```

The floor stays fixed. Main placement depends on where the sale ends:

| Remaining progress        | New main offset $`b'`$           |
| ------------------------- | -------------------------------- |
| $`Q\ge E`$                | $`b`$                            |
| $`A\lt Q\lt E`$           | $`b+(k-1)\bigl(g(Q)-g(E)\bigr)`$ |
| $`Q\lt E`$ and $`Q\le A`$ | $`F-g(Q)`$                       |

At the old ramp start, the second and third expressions agree for a continuous
join. If the old ramp already has zero width and $`Q=E`$, geometry is unchanged.

### Sale ending inside the ramp

The retained ramp remains the same function on $`[A,Q]`$. The new main attaches
at its endpoint: $`M'(Q)=R(Q)`$. Thus the sale lowers main placement, preserves
the retained exact backing, and makes the next purchase start on main at $`Q`$.
The characteristic function itself does not change.

### Sale ending in the floor

The ramp collapses to $`A'=E'=Q`$. Main becomes

```math
M'(x)=g(x)+F-g(Q),
```

so $`M'(Q)=F`$. Remaining circulation is backed at the floor until subsequent
purchases add a new main tail. If floor operations previously made $`T\ne P`$,
zero circulation need not mean zero progress.

### Numerical contraction

A fixed-point calculation uses an upper bound for the new offset to keep the
retained schedule from falling below its required floor. Any additional retained
backing is conservatively deducted from the sale's proceeds. This is separate
from the redemption fee. The exact equations above need no such deduction.

## Floor raise

A raise increases $`F`$ and supplies new ramp geometry. It preserves $`b`$,
$`P`$, $`T`$, and the immutable characteristic and multiplier. It requires

```math
A'\ge A,\qquad E'\ge E,\qquad A'\le E'\le P.
```

The floor tail becomes $`J'=T-P+A'`$. The live price stays $`M(P)`$ because main
placement is unchanged and the new ramp ends at or behind progress.

An ordinary raise preserves backing, subject to a one-sided numerical allowance.
A [floor investment](Floor-Investment) may increase backing using existing
surplus. Neither action transfers reserve or mints AVA. Increasing the floor
does not imply that ramp width increases; the proposed joins and backing must be
checked directly.

## Floor issuance and floor redemption

For authorized [option exercise](Call-Options) of $`n`$ AVA:

```math
T'=T+n,\qquad J'=J+n,\qquad B'=B+Fn.
```

The holder supplies $`Fn`$ plus the exercise fee. Progress and the entire
schedule stay fixed. There is no translation of the characteristic or ramp
boundaries.

For special floor redemption of $`0\lt n\le J`$:

```math
T'=T-n,\qquad J'=J-n,\qquad B'=B-Fn.
```

The holder receives $`Fn`$ before the redemption fee. Progress and geometry
again stay fixed. Holdings, collateral, and liquid reserve must support the
operation.

## Reserve advances and donations

An advance draw of $`d`$ exchanges held reserve for a collateralized advance:
$`H'=H-d`$, $`D'=D+d`$. Repayment reverses this exchange. These operations
preserve $`H+D`$, $`P`$, $`T`$, and schedule geometry. The draw must satisfy
collateral and liquidity checks; a reserve-access fee is separate from backing.

A donation of $`v`$ sets $`H'=H+v`$ and leaves required backing unchanged. It
creates surplus available for a later authorized floor investment.

## What each action changes

| Action                   | Progress $`P`$                   | Circulation $`T`$ | Floor $`F`$ | Main offset $`b`$         | Required backing                           |
| ------------------------ | -------------------------------- | ----------------- | ----------- | ------------------------- | ------------------------------------------ |
| Ordinary purchase        | Increases                        | Increases         | Fixed       | Fixed                     | Increases                                  |
| Ordinary redemption      | Decreases, bounded below by zero | Decreases         | Fixed       | May change on contraction | Does not increase                          |
| Ordinary floor raise     | Fixed                            | Fixed             | Increases   | Fixed                     | Preserved, with bounded numerical decrease |
| Floor investment         | Fixed                            | Fixed             | Increases   | Fixed                     | May increase                               |
| Floor issuance           | Fixed                            | Increases         | Fixed       | Fixed                     | Increases by $`Fn`$                        |
| Special floor redemption | Fixed                            | Decreases         | Fixed       | Fixed                     | Decreases by $`Fn`$                        |
| Advance or repayment     | Fixed                            | Fixed             | Fixed       | Fixed                     | Fixed                                      |
| Donation                 | Fixed                            | Fixed             | Fixed       | Fixed                     | Fixed                                      |

A contraction can reduce the progress threshold from which further trading and
floor raises are possible. A future floor increase still requires a valid,
funded proposal; the trading cycle does not guarantee one.

<!--
title: Call Options
summary:
  Authorized issuance at the current floor, with unchanged pricing progress and explicit backing.
tags:
  - avm-features
  - options
-->

An application can grant an option to acquire AVA at the market's **current
floor**. The AVM supplies the floor-issuance accounting operation. Who receives
an option, whether it expires or transfers, and how exercise rights are consumed
are application rules. The pricing operation alone grants no exercise right.

## Exercise

Use $F$ for floor price, $P$ for pricing progress, $T$ for circulation,
$J=T-P+A$ for the physical floor tail, $B$ for required backing, and $H$ for
held reserve, as in the [AVM specification](AVM-Specification).

An authorized exercise for $n>0$ AVA has the exact changes

$$
T'=T+n,\qquad J'=J+n,\qquad B'=B+Fn,\qquad H'=H+Fn.
$$

The holder pays $Fn$ plus the configured exercise fee and receives $n$ AVA.
Progress $P$, floor $F$, ramp boundaries, and main offset all stay fixed.
Consequently, the live price $M(P)$ stays fixed.

There is no rightward translation of the stored curve. Separating circulation
from progress accounts for the additional floor-backed supply directly.

## Backing and settlement

The added backing requirement is exactly $Fn$ in real arithmetic. The payment
covers that requirement; an exercise does not create an unfunded floor claim.
With bounded arithmetic, the reserve charge is rounded conservatively upward.
The holder's maximum reserve payment includes the exercise fee.

The application must verify and consume the exercise entitlement, collect reserve,
issue the correct AVA amount, distribute fees, and commit the next state in one
atomic settlement.

## Economic effect

If the live price exceeds the floor, an exercise right can have value. Immediate
ordinary redemption of the new AVA pays the integral over the sale interval,
less redemption fees and any conservative numerical adjustment. For a small
quantity its gross per-unit proceeds approach the live price; a large quantity
has price impact. Profit cannot be computed as spot price times quantity without
accounting for that impact.

An exercise fee reduces the value of the right but does not by itself control
how many rights exist. Granting unrestricted floor issuance would make those
rights available to anyone and materially alter the market's economics. An
application must define admission and issuance limits deliberately.

Exercise preserves backing at that moment, but it adds circulating claims and
increases the backing needed for later floor raises. Existing holders retain
their floor claim; the distribution of future price-discovery value can change.

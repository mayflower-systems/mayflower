<!--
title: Linear AVM Floor Investment
summary:
  Exact backing increases for linear surplus-funded floor raises.
tags:
  - linear-avm
  - floor
  - math
-->

[Floor investment](Floor-Investment) accepts a higher floor when existing
surplus covers the increase in required backing. For the linear market, use
main slope $m$, ramp slope $a>m$, main intercept $b$, floor $F$, joins $A,E$,
progress $P$, circulation $T$, and $h=T-P$.

## General candidate

The proposal preserves $m,a,b,P,T$ and chooses $F'>F$, $A'$, and $E'$ with

$$
A'\ge A,\qquad E'\ge E,\qquad 0\le A'\le E'\le P.
$$

Exact continuity requires

$$
F'=aA'+b-(a-m)E'.
$$

The resulting backing increase is

$$
\Delta B=h(F'-F)
 +\frac{a(A'^2-A^2)-(a-m)(E'^2-E^2)}2.
$$

It must be nonnegative, no larger than the permitted reserve budget, and covered
by available surplus $H+D-B$. Numerical bounds and the allowed upward join gap
follow the [general floor-raise rules](AVM-Floor-Raising).

## Fixed-end special case

Holding $E'=E$ also holds the ramp intercept $c=b-(a-m)E$ fixed. The higher
floor shortens the ramp from below:

$$
A'=\frac{F'-c}{a},\qquad
\Delta B=h(F'-F)+\frac{(F'-c)^2-(F-c)^2}{2a}.
$$

The maximum floor for this fixed-end shape is $F'=mE+b$, at which $A'=E$ and
the ramp has zero width. Funding alone does not authorize raising above that
main price. Other candidates may advance $E'$ up to $P$.

## Worked example

Use $m=8$, $a=25/2$, $b=1$, $F=16$, $A=3$, $E=5$, and $P=T=6$. Backing is
150 and spot price is 49. The ramp intercept is $c=-43/2$.

Raise the floor to 21 while keeping $E=5$. Then

$$
A'=\frac{21+43/2}{25/2}=\frac{17}{5},\qquad
W'=5-\frac{17}{5}=\frac85.
$$

Because $h=0$,

$$
\Delta B=\frac{25}{4}\left(\frac{289}{25}-9\right)=16.
$$

The proposed backing is 166. A donation of 16 to an exactly funded market
supplies the needed surplus. After the accepted raise, main remains $8x+1$
and spot stays 49. The ramp has narrowed from width 2 to $8/5$.

If circulation were instead 8 with the same old geometry and progress, the
backing increase would be $16+(8-6)(21-16)=26$. The extra floor-backed units
must receive the same higher floor.

## What the donation changes

Donation increases held reserve and leaves the schedule unchanged. Investment
then increases required backing without transferring reserve. Keeping these
steps separate makes both the funding and the accepted geometry explicit.
The amount of a donation does not select a unique floor or ramp shape.

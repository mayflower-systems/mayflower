<!--
title: Linear AVM Evolution
summary:
  Linear state transitions for trading, contraction, and floor operations.
tags:
  - linear-avm
  - math
-->

This page specializes [AVM Evolution](AVM-Evolution) to the
[linear characteristic](Linear-AVM-Specification). Use floor $`F`$, main slope
$`m\gt 0`$, ramp slope $`a\gt m`$, main intercept $`b`$, ramp intercept
$`c=b-(a-m)E`$, joins $`A,E`$, progress $`P`$, circulation $`T`$, and floor tail
$`J=T-P+A`$.

## Purchase

An ordinary purchase of $`n`$ AVA pays

```math
C=n\left(mP+b+\frac{mn}{2}\right)
```

before fees and sets $`P'=P+n`$, $`T'=T+n`$. Geometry and both intercepts stay
fixed. The purchase starts on main because $`P\ge E`$.

## Sale ending on main

Let $`Q=\max(P-n,0)`$. If $`n\le P`$ and $`Q\ge E`$, the gross sale proceeds are

```math
V=n\left(mP+b-\frac{mn}{2}\right).
```

The sale lies entirely on main. Set $`P'=Q`$, $`T'=T-n`$ and leave geometry
unchanged.

## Sale ending inside the ramp

If $`A\lt Q\lt E`$, quote the sale over the old ramp and main intervals, then set

```math
A'=A,\quad E'=Q,\quad W'=Q-A,\quad
b'=b+(a-m)(Q-E).
```

The new main connects at the sale endpoint:

```math
mQ+b'=aQ+c.
```

Its intercept decreases, while the retained ramp intercept remains $`c`$.
Progress becomes $`Q`$ and circulation becomes $`T-n`$. The next purchase
follows the new main from $`Q`$.

## Sale ending in the floor

If $`Q\lt E`$ and $`Q\le A`$, quote the sale on the old schedule and then collapse
the ramp:

```math
A'=E'=Q,\quad W'=0,\quad b'=F-mQ,\quad c'=F-aQ.
```

The new live price is $`mQ+b'=F`$. The floor is unchanged. The intercept $`b'`$
can be higher or lower than the old intercept; the defining rule is attachment
at $`(Q,F)`$, not a universal downward shift of the entire main line.

With circulation different from progress, a complete redemption can leave
$`T'=0`$ but positive $`P'=Q`$.

## Sale exceeding progress

When floor issuance has made $`T\gt P`$, an ordinary sale can have $`P\lt n\le T`$. It
pays

```math
V=\int_0^P y(x)\,dx+F(n-P)
```

before fees, sets $`P'=0`$ and $`T'=T-n`$, and applies the same contraction
rules. If $`E\gt 0`$, the ramp collapses at zero. If $`E=0`$, geometry is
unchanged; the sale still needs the separate $`F(n-P)`$ term rather than the
main-only formula.

## Ordinary floor raise

A raise preserves $`m,a,b,P,T`$, increases $`F`$, and selects $`A',E'`$ with
$`A'\ge A`$, $`E'\ge E`$, and $`A'\le E'\le P`$. Exact continuity gives

```math
c'=b-(a-m)E',\qquad F'=aA'+c'.
```

Required backing stays fixed in the exact area-preserving model. The new ramp
width may increase or decrease.
[Linear AVM Floor Raising](Linear-AVM-Floor-Raising) gives the constant-backing
solution, including the case $`T\ne P`$.

## Floor investment

The same boundary and join rules apply, with main intercept $`b`$ still
unchanged. The difference is the backing policy: a nonnegative increase must fit
the permitted budget and available surplus. A fixed-end investment can narrow
the ramp to zero width. See
[Linear AVM Floor Investment](Linear-AVM-Floor-Investment).

## Floor issuance and special floor redemption

Authorized issuance of $`n`$ AVA at the floor sets

```math
T'=T+n,\quad J'=J+n,\quad B'=B+Fn.
```

Special floor redemption of $`n\le J`$ sets

```math
T'=T-n,\quad J'=J-n,\quad B'=B-Fn.
```

Both leave $`P,F,A,E,b,c`$ unchanged. No line translates and no boundary moves.
The corresponding reserve payment, fees, authorization, and collateral checks
must settle with the state change.

## Example of contraction

Start from $`m=8`$, $`a=25/2`$, $`b=1`$, $`F=16`$, $`A=3`$, $`E=5`$, $`P=T=6`$.
Sell two AVA, leaving $`Q=4`$ inside the ramp. Gross proceeds on the old
schedule are 45 from $`[5,6]`$ plus $`(57/2+41)/2=139/4`$ from $`[4,5]`$,
totaling $`319/4`$.

The new main intercept is

```math
b'=1+\frac92(4-5)=-\frac72,
```

so its price at progress four is $`8\cdot4-7/2=57/2`$, equal to the old ramp
price there. Required backing becomes $`150-319/4=281/4`$. The floor stays 16.
These amounts exclude fees and fixed-point rounding.

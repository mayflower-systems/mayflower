<!--
title: Linear AVM Corollaries
summary:
  Derived price-impact, capital, and floor-limit relationships with explicit assumptions.
tags:
  - linear-avm
  - math
-->

These are consequences of the [linear AVM specification](Linear-AVM-Specification),
not additional operating rules. They use exact arithmetic and exclude fees,
advances, floor changes, and external exchange-rate movements unless stated.

## Price impact on a single linear segment

For a segment with slope $s\ge0$, starting price $p_0$, and positive quantity
$n$, moving forward by $n$ changes the price by $sn$ and costs

$$
C=n\left(p_0+\frac{sn}{2}\right).
$$

The average price is $p_0+sn/2$. A sale moving backward by $n$ has gross average
price $p_0-sn/2$, provided it stays on that segment. Cross-segment sales require
summing separate integrals.

In the AVM, ordinary purchases begin on main, whose slope is $m$. Redemptions can
cross the ramp, whose slope is $a>m$, and floor, whose slope is zero. The ramp's
larger slope describes greater price sensitivity per AVA within that interval;
it does not guarantee higher returns or measure statistical volatility.

## Cost to double a positive price

On an unchanged linear segment with slope $s>0$ and starting price $p_0>0$,
reaching $2p_0$ requires $n=p_0/s$. The cost is

$$
K=\frac{3p_0^2}{2s},\qquad s=\frac{3p_0^2}{2K}.
$$

For ordinary issuance, use $s=m$ and the current main price for $p_0$.
For example, $p_0=0.1$ and $K=100$ imply $m=0.00015$ and
$n=666.\overline6$. These values concern movement along that unchanged main
line; they do not calibrate every later market state automatically.

## Price multiplier versus capital

Starting at $p_0$ on the same unchanged line, capital inflow $C$ yields spot
price $p_1$ satisfying

$$
p_1^2=p_0^2+2sC.
$$

Using the cost-to-double $K$ above,

$$
\frac{p_1}{p_0}=\sqrt{1+\frac{3C}{K}},\qquad
\frac CK=\frac{(p_1/p_0)^2-1}{3}.
$$

| Spot-price multiplier | Required $C/K$ |
| --------------------- | -------------- |
| $2$                   | $1$            |
| $5$                   | $8$            |
| $10$                  | $33$           |
| $50$                  | $833$          |
| $100$                 | $3333$         |

These are **spot-price multipliers**, not realized investment returns. A buyer
pays an average fill price and a seller receives an average fill price; both
can differ from the marginal spot quote. Fees and subsequent state changes
further affect returns.

## Ramp geometry

For a given endpoint, the ramp lies below main before their intersection. Its
purpose in floor raising is to provide a lower-area transition that can replace
part of main while maintaining the endpoint price.

This is not an early-buyer ramp: launch has zero ramp width, and after every
accepted ordinary redemption the next purchase begins on main. Applying the
ramp's slope to estimate ordinary buy-driven returns would describe a path the
market does not take.

## Maximum constant-backing floor

There is no universal floor-to-spot percentage. The bound depends on existing
geometry and on both circulation and progress. The complete derivation is in
[Linear AVM Floor Raising](Linear-AVM-Floor-Raising).

For the special unraised state with $A=E=0$, $T=P$, and $F=b$, main slope $m$
and ramp multiplier $k=a/m$ give

$$
F_{\max}=b+mP\left(\sqrt{k(k-1)}-(k-1)\right).
$$

At $k=2$, the increase above $b$ is approximately $0.4142mP$. The fraction of
the full spot price $mP+b$ depends on $b$ as well. Prior floor operations require
the general state-dependent expression.

## Floor backing is finite

The floor component is $FT$ in the exact backing decomposition. It supports a
minimum gross reserve claim, subject to collateral obligations. Special
price-preserving floor redemption is limited by the physical floor tail
$J=T-P+A$ and available liquid reserve. A zero slope is not infinite capital
or an unlimited withdrawal right.

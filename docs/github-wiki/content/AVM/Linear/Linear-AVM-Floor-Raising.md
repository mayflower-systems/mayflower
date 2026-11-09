<!--
title: Linear AVM Floor Raising
summary:
  Closed-form constant-backing geometry, maximum-floor conditions, and verified examples.
tags:
  - linear-avm
  - floor
  - math
-->

This page derives an exact area-preserving raise for the
[linear AVM](Linear-AVM-Specification). Let $m>0$ be the main slope, $a>m$ the
ramp slope, $b$ the main intercept, $F$ the floor, $A$ the ramp start, $E$ the
ramp end, $P$ progress, and $T$ circulation. Define

$$
s=a-m,\qquad h=T-P,\qquad J=h+A.
$$

The derivation assumes exact continuous joins and preserves $P,T,b,m,a$.
Fixed-point proposals must still pass the [general numerical checks](AVM-Floor-Raising).

## Backing equation

The required backing is

$$
B=\frac{mP^2}{2}+bP+hF+\frac{aA^2-sE^2}{2}.
$$

Since the floor join gives $F=aA+b-sE$, preserving $B$ is equivalent to

$$
a(A'^2-A^2)-s(E'^2-E^2)+2h(F'-F)=0.
$$

The term with $h$ cannot be omitted after floor issuance or special floor
redemption.

## Solve for a chosen ramp end

Choose $E'\in[E,P]$. Completing the square gives

$$
J'=\sqrt{J^2+\frac{s}{a}\left((E'+h)^2-(E+h)^2\right)},
$$

$$
A'=J'-h,\qquad
F'=aA'+b-sE',\qquad W'=E'-A'.
$$

The nonnegative square root is selected because $J'$ is a physical quantity.
A valid positive raise must have $F'>F$ and satisfy the join and boundary rules.
The derived ramp intercept is $c'=b-sE'$. The ramp width need not increase.

When circulation equals progress ($h=0$), this simplifies to

$$
A'=\sqrt{A^2+\frac{s}{a}(E'^2-E^2)},\qquad
F'=aA'+b-sE'.
$$

## Maximum floor at fixed progress and circulation

Along this exact constant-backing family, where $J'>0$,

$$
\frac{dF'}{dE'}=s\frac{E'-A'}{J'}\ge0.
$$

Thus the largest continuous floor in the family is attained at the largest
permitted end, $E'=P$. Cases with zero tail follow by continuity and must still
satisfy the state constraints. Substituting $E'=P$ into the formulas above gives
the bound for the current state. It is not a fixed percentage of spot price.

A useful special case starts from an unraised launch schedule with
$A=E=0$, $T=P$, and $F=b$. Then

$$
F_{\max}=b+P\left(\sqrt{a(a-m)}-(a-m)\right).
$$

For $a=2m$, the floor increase is $(\sqrt2-1)mP$, approximately 41.42% of
the live price's increase above $b$. The full ratio $F_{\max}/(mP+b)$ also
depends on $b$. This formula does not apply unchanged to a previously raised
market or a state with $T\ne P$.

## Worked example

Take $m=8$, $a=25/2$, $b=F=1$, $A=E=0$, and $P=T=6$. The initial backing is
150 reserve units and spot price is 49.

Choose $E'=5$. Since $s/a=(9/2)/(25/2)=9/25$,

$$
A'=\sqrt{\frac9{25}\cdot25}=3,
\qquad
F'=\frac{25}2\cdot3+1-\frac92\cdot5=16.
$$

The new width is $W'=2$. Backing is

$$
16\cdot3+\frac{16+41}{2}\cdot2+\frac{41+49}{2}\cdot1=150.
$$

For the maximum permitted $E'=6$, the same calculation gives

$$
A'=\frac{18}{5},\qquad W'=\frac{12}{5},\qquad F'=19.
$$

Backing remains $19(18/5)+(19+49)(12/5)/2=150$, and spot stays 49.

## Why circulation matters

Suppose two AVA are issued at the floor after the first raise to 16. Then
$T=8$, $P=6$, and backing rises from 150 to 182. Reusing the former maximum
candidate $F'=19,A'=18/5,E'=6$ would require

$$
B'=150+(8-6)19=188.
$$

That is a six-unit increase, so this candidate is no longer area-preserving.
It could only be admitted under a suitable surplus-funded policy. A new
constant-backing solution must use $h=2$ in the full formula.

---
title: Specification
summary:
  General definition of the AVM price curve — three monotonic segments (floor,
  ramp, main), the ramp constraint, boundary points, the solvency invariant,
  buying, selling, contraction, and the guaranteed-vs-discovery value
  decomposition.
tags:
  - avm
  - pricing
  - math
---

The AVM price curve is a continuous, monotonically non-decreasing function of
supply that determines the price of every buy and every sell. This note defines
the curve in terms of general monotonic functions — the constraints any
conforming implementation must satisfy — without specifying a particular
functional form.

---

## Definition

The price curve $y(x)$ has three segments:

$$
y(x) = \begin{cases}
f & 0 \le x \le x_1 \quad \text{(floor)} \\[4pt]
R(x) & x_1 < x < x_2 \quad \text{(ramp)} \\[4pt]
M(x) & x_2 \le x \quad \text{(main)}
\end{cases}
$$

**Floor.** A constant function at height $f$. This is the guaranteed minimum
redemption price for every token in circulation. The floor can only increase,
never decrease.

**Ramp.** A strictly increasing function $R(x)$ on the interval $(x_1, x_2)$.
The ramp is the transition zone between the floor guarantee and the open market.
It is steeper than the main schedule — early capital moves price faster here.

**Main.** A strictly increasing function $M(x)$ on $[x_2, \infty)$. This is the
long-run pricing schedule where the market spends most of its life.

**Continuity.** The curve is continuous at both boundaries:

$$
R(x_1) = f, \qquad R(x_2) = M(x_2)
$$

The ramp meets the floor at $x_1$ and meets the main at $x_2$. There are no
jumps.

---

## The Ramp Constraint

The ramp function must satisfy one critical structural property:

$$
R(x) < M(x) \quad \text{for all } x \in (x_1, x_2)
$$

In the ramp region, the ramp sits **below** the main function. Both are
increasing functions that meet at $x_2$, but the ramp approaches from below — it
rises steeply from the floor while the main, extrapolated backward, would be at
a higher level throughout the transition zone.

This is the property that makes floor raising possible. When the ramp–main
boundary $x_2$ advances, a strip of the curve that was priced on the main
schedule is replaced by ramp. Because the ramp is lower than the main in that
strip, the area under the curve **decreases**. This decrease is what funds the
higher floor.

Without this constraint, there is no area deficit to redistribute, and floor
raising cannot preserve solvency.

---

## Boundary Points

$x_1$ and $x_2$ are not free parameters — they are determined by the floor level
and the shapes of $R$ and $M$:

- $x_1$ is where the ramp meets the floor: $R(x_1) = f$.
- $x_2$ is where the ramp meets the main: $R(x_2) = M(x_2)$.

When the floor rises, $x_1$ and $x_2$ move as a consequence. At launch, both
boundaries coincide ($x_1 = x_2$) and the ramp has zero width. Floor raises are
what create and widen the ramp.

---

## Initial Conditions

At launch:

$$
x_1 = x_2 = 0, \quad x = 0, \quad R = 0
$$

The floor and ramp have zero width. The entire curve is the main schedule $M(x)$
starting from the initial floor price $f_0$. The first buy walks up this
schedule. The ramp does not exist as a distinct segment until a floor raise
separates the boundaries.

---

## Reserve and Solvency

The **reserve** is the total capital backing all tokens in circulation. It
equals the area under the price curve from $0$ to the current supply:

$$
R(x) = \int_0^x y(s)\, ds
$$

This is the **solvency invariant**: the reserve holds enough capital to redeem
every token at the price the curve assigns. No token is unbacked. This invariant
is maintained through every operation — buys, sells,
[[avm--floor-raising|floor raises]], and
[[avm-features--call-options|option executions]].

---

## Buying (Issuance)

A buy increases supply from $x$ to $x' > x$. The buyer pays the integral of the
curve over the new region:

$$
\text{cost} = \int_x^{x'} y(s)\, ds
$$

This capital is added to the reserve. The buyer's fill price is the average:

$$
\bar{y} = \frac{1}{x' - x} \int_x^{x'} y(s)\, ds
$$

Buys do not alter the curve — they move supply along the existing schedule. The
shape of $R$, $M$, and the positions of $x_1$, $x_2$ are unchanged.

---

## Selling (Redemption)

A sell decreases supply from $x$ to $x' < x$. The seller receives the integral:

$$
\text{proceeds} = \int_{x'}^{x} y(s)\, ds
$$

This capital is removed from the reserve.

If the sell keeps supply at or above $x_2$, the curve is unchanged — the sell
simply moves supply down the main schedule.

---

## Contraction

When a sell pushes supply below $x_2$, the curve must adapt. There are two
cases.

**Sell into ramp** ($x_1 \le x' < x_2$). The ramp is unchanged. The main
schedule shifts down so that it meets the ramp at the new, lower supply. The
ramp–main boundary contracts: $x_2$ moves back to the new supply.

**Sell into floor** ($x' < x_1$). The ramp collapses to zero width. Both the
ramp and main schedules re-anchor at the new supply point at height $f$, so
$x_1 = x_2 = x'$. If supply returns all the way to $0$, the curve resets to its
launch shape (though $f$ may have been raised by prior floor raises).

In both cases, contraction is permanent with respect to buying: subsequent buys
walk up the shifted curve without restoring the old main schedule. Only a floor
raise can advance $x_2$ again.

See [[avm--evolution|Evolution]] for the complete walkthrough of every state
change, and [[avm--floor-raising|Floor Raising]] for the area-preserving
mechanism that advances $x_2$.

---

## Properties

### Floor as Guaranteed Redemption

In the floor region ($0 \le x \le x_1$), the price is constant at $f$:

- Selling into the floor has **zero price impact** — the floor provides infinite
  sell-side liquidity.
- The floor price is the guaranteed minimum redemption value for every token,
  backed by the reserve.
- The floor can only increase (via [[avm--floor-raising|floor raises]] or
  [[avm-features--floor-investment|floor investment]]), never decrease.

### Market Value and Guaranteed Value

Every token has two values: the **market price** $y(x)$ at the current supply,
and the **guaranteed value** $f$ (the floor). The difference is the token's
exposure to price discovery.

The area under the curve decomposes into two regions:

- **Guaranteed region** — the rectangle from $0$ to supply at height $f$. This
  is $f \cdot x$: the total value backed unconditionally. Every token can be
  redeemed for at least $f$ regardless of what other participants do.
- **Discovery region** — the area above $f$, between $x_1$ and supply. This is
  where all price volatility lives. It exists only because supply is above $x_1$
  — it is created by demand and destroyed by sells.

Floor raising converts discovery-region area into guaranteed-region area. The
total reserve is unchanged, but more of it becomes unconditional. After a raise,
the floor is higher, the volatile pocket is smaller, and each token's guaranteed
value has increased.

### Sensitivity

The steepness of the active segment determines the market's price sensitivity to
supply changes:

- **Floor**: zero sensitivity — infinite sell-side liquidity.
- **Ramp**: high sensitivity — price moves rapidly. This is the transition zone
  between the floor guarantee and the open market.
- **Main**: moderate sensitivity — the long-run regime where the market spends
  most of its life. Designed for stability.

The ramp is steeper than main by construction. This is not incidental — it is
the structural property that makes floor raising possible. A single uniform
pricing schedule cannot support area-preserving floor raises; the two-schedule
design (steep ramp, gentle main) is the minimum structure required.

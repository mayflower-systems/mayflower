<!--
title: Linear AVM Specification
summary:
  The piecewise-linear implementation of the AVM — three line segments (floor,
  ramp, main) defined by two immutable slopes and three state variables, with
  closed-form expressions for intersection points, contraction, floor raising,
  and every fill price.
tags:
  - linear-avm
  - math
-->

This is the piecewise-linear implementation of the
[general AVM specification](AVM-Specification). The ramp and main are
instantiated as linear functions, which gives closed-form expressions for every
intersection point, contraction, floor raise, fill price, and reserve balance.
The five parameters $(f, b_1, b_2, m_1, m_2)$ fully determine the curve.

## Definition

The price curve $y(x)$ is a piecewise-linear, continuous, monotonically
non-decreasing function of supply $x$, with three segments:

$$
y(x) = \begin{cases}
f & 0 \le x < x_1 \quad \text{(floor)} \\[4pt]
m_1\, x + b_1 & x_1 \le x < x_2 \quad \text{(ramp)} \\[4pt]
m_2\, x + b_2 & x_2 \le x \quad \text{(main)}
\end{cases}
$$

**Immutable parameters** (set at market creation, never change):

- $m_1$ — slope of the ramp ($m_1 > m_2$; typically $m_1 \approx 2\, m_2$)
- $m_2$ — slope of main ($m_2 > 0$)

**State variables** (evolve via floor raises, sell contractions, and option
executions):

- $f$ — floor price (the constant of the floor line)
- $b_1$ — y-intercept of the ramp line
- $b_2$ — y-intercept of the main line

These are the y-intercepts of three lines. The curve is fully determined by
$(f, b_1, b_2)$ plus the two immutable slopes.

**Other:**

- $x$ — total supply of m-tokens (changes on every buy/sell, but does not affect
  the curve shape)
- $y$ — price (exchange rate of the m-token in terms of the quote token)

The curve parameters are static most of the time. They only change during a
floor raise (area-preserving or
[surplus-funded](Floor-Investment)), a sell that pushes supply
below $x_2$, or an [option execution](Call-Options). All other
operations (buys, sells within the main segment) move supply along the existing
curve without altering it.

### Intersection points $x_1$ and $x_2$

The curve is three lines. They intersect at two points:

$x_1$ is where the floor meets the ramp ($f = m_1\, x_1 + b_1$):

$$
x_1 = \frac{f - b_1}{m_1}
$$

$x_2$ is where the ramp meets the main ($m_1\, x_2 + b_1 = m_2\, x_2 + b_2$):

$$
x_2 = \frac{b_2 - b_1}{m_1 - m_2}
$$

$x_1$ and $x_2$ are not free parameters — they are determined by the three
y-intercepts and the two slopes. When the state variables change, the
intersection points move as a consequence.

At launch ($b_1 = b_2 = f$), both intersection points are at $0$ and the ramp
has zero width. As floor raises decrease $b_1$ (while holding $b_2$ fixed), both
$x_1$ and $x_2$ advance and the ramp opens up. During sell contractions, $b_2$
drops (while $b_1$ stays), pulling $x_2$ back. If supply drops below $x_1$, both
lines re-anchor through the new supply at height $f$, collapsing the ramp to
zero width.

## Initial Conditions

At launch, all three lines coincide at the origin:

$$
f = b_1 = b_2 = f_0, \quad x = 0, \quad R = 0
$$

Since $b_1 = b_2 = f$, both intersection points are at $0$ ($x_1 = x_2 = 0$).
The floor and ramp have zero width. The entire curve is the main ray:

$$
y = m_2\, x + f \quad \text{for all } x \ge 0
$$

The first buy walks up this line from $f$. The ramp and floor do not exist as
distinct segments until a floor raise separates the y-intercepts.

The immutable parameters $m_1$ and $m_2$ are set at creation. The initial floor
price $f$ is also set at creation but can be raised later.

## Reserve and Solvency

The **reserve** is the total capital backing all m-tokens in circulation. It
equals the area under the price curve from $0$ to the current supply $x$:

$$
R(x) = \int_0^x y(s)\, ds
$$

This is the solvency invariant: at any point, the reserve holds enough capital
to redeem every m-token in existence at the price the curve assigns. No m-token
is unbacked.

## Buying (Issuance)

A buy increases supply from $x$ to $x' > x$. The buyer pays the integral of the
curve over the new region:

$$
\text{cost} = \int_x^{x'} y(s)\, ds
$$

This capital is added to the reserve. The buyer's **fill price** (average price
per token) is:

$$
\bar{y} = \frac{1}{x' - x} \int_x^{x'} y(s)\, ds
$$

The spot price after the buy is $y(x')$. Buys do not move $x_2$ — the curve
shape is unchanged. The buy simply advances supply along the existing curve.

## Selling (Redemption)

A sell decreases supply from $x$ to $x' < x$. The seller receives the integral:

$$
\text{proceeds} = \int_{x'}^{x} y(s)\, ds
$$

This capital is removed from the reserve.

### Contraction Rule

When a sell pushes supply below $x_2$, the main line shifts down to pass through
the ramp at the new supply point. There are two cases:

**Sell into ramp** ($x_1 \le x^{\,\text{new}} < x_2$): The ramp line does not
move ($b_1$ unchanged). The main line re-anchors through the ramp at the new
supply, so $b_2$ **decreases**:

$$
b_2^{\,\text{new}} = (m_1 - m_2)\, x^{\,\text{new}} + b_1
$$

Since $b_1$ is unchanged, $x_1$ does not move. But $b_2$ dropped, so the
intersection $x_2 = (b_2 - b_1)/(m_1 - m_2)$ contracts to the new supply. The
ramp gets narrower but does not disappear.

**Sell into floor** ($x^{\,\text{new}} < x_1$): The ramp collapses to zero
width. Both lines re-anchor through $(x^{\,\text{new}},\, f)$:

$$
b_1^{\,\text{new}} = f - m_1\, x^{\,\text{new}}, \quad b_2^{\,\text{new}} = f - m_2\, x^{\,\text{new}}
$$

Now $x_1 = x_2 = x^{\,\text{new}}$. In the limit where supply returns to $0$,
$b_1 = b_2 = f$ and the curve resets to its launch shape (though $f$ may have
been raised by prior floor raises).

In both cases, contraction is permanent with respect to buying: subsequent buys
walk up the existing curve without restoring the old $b_2$. Only a floor raise
can advance $x_2$ again.

## Floor Raising

Floor raising is the mechanism that evolves the curve. It increases $f$ (the
floor price), which slides $x_2$ forward and opens up the ramp behind the
current supply.

### What changes

- $f$ increases (the floor line rises).
- $b_1$ **decreases** (the ramp line shifts to reach the higher floor while
  staying connected to the unchanged main line).
- **$b_2$ does not change.** The main line stays exactly where it is.

Since $b_2$ is fixed and $b_1$ decreased, $x_2 = (b_2 - b_1)/(m_1 - m_2)$
increases — the intersection slides forward along the main line. Since $f$
increased and $b_1$ decreased, $x_1 = (f - b_1)/m_1$ increases too. The ramp
widens.

### Invariants

- $b_2$, $m_1$, $m_2$ — the main line does not move.
- Reserve — the area under the curve from $0$ to supply is unchanged.
- Price at supply — the spot price does not move.

### Why the invariant holds

The ramp has a steeper slope than main ($m_1 > m_2$). Advancing the ramp–main
intersection replaces a region that was priced on the main line (gentle slope)
with ramp (steep slope), which **decreases** the area under the curve in that
region. This decrease exactly compensates for the **increase** in area from
raising the floor. The reserve stays the same — the curve is still solvent.

### Precondition and magnitude

A floor raise requires $\text{supply} > x_2$. The supply must be in the main
region — there must be room to slide $x_2$ forward.

The magnitude of the raise is bounded by geometry: the new $x_2$ cannot exceed
the current supply (otherwise the price at supply would change, violating the
invariant). The further supply sits beyond $x_2$, the more room there is to
raise. If supply is far above $x_2$, the floor could double or more in a single
raise — there is no incremental limit, only the area-preservation constraint.

### Sell contraction enables future floor raises

Selling into the ramp or floor lowers $b_2$, which pulls $x_2$ back. This
**lowers the bar** for future floor raises: once $x_2$ has been pushed back, any
subsequent buy that puts supply above $x_2$ re-enables floor raising —
potentially at a lower price level than was possible before.

In the extreme case where supply is sold all the way back to $0$
($b_1 = b_2 = f$, $x_1 = x_2 = 0$), the very next buy creates supply beyond
$x_2$, enabling a floor raise immediately. This means sell pressure does not
permanently damage the floor mechanism — it can actually create the conditions
for a higher floor afterward.

## Slopes Are Immutable

The slopes $m_1$ and $m_2$ never change. The curve evolves only through four
mechanisms:

- **Floor raising** (area-preserving) — $f$ increases, $b_1$ decreases, $b_2$
  unchanged. The main line stays put; the ramp adjusts. Reserve preserved.
- **Floor investment** (surplus-funded raise) — $f$ increases and the ramp may
  shorten or collapse. $x_2$ may remain fixed or advance past its old position.
  The live price is preserved and any increase in required reserve must be
  covered by effective reserve.
- **Sell contraction** — $b_2$ decreases, $b_1$ unchanged (sell into ramp) or
  both adjust (sell into floor). The main line shifts down; the ramp stays put.
  Capital leaves the reserve.
- **Option execution** — $b_1$ decreases by $m_1 n$, $b_2$ decreases by $m_2 n$,
  $f$ unchanged. Both lines shift right by $n$, widening the floor. Reserve
  preserved.

See [Evolution](Linear-AVM-Evolution) for a complete walkthrough of each case.

## Properties

### Price Impact

Because $y(x)$ is piecewise-linear, price impact is deterministic and computable
from the slopes alone. Given a buy of $\Delta x$ tokens starting at supply $x$:

- If $x$ and $x + \Delta x$ are in the same segment with slope $m$, the price
  moves by exactly $m \cdot \Delta x$.
- If the buy crosses a segment boundary, the impact is the sum of the
  contributions from each segment.

### Floor as Buy Wall

In the floor region ($0 \le x < x_1$), the slope is zero:

- Selling into the floor has **zero price impact** — the floor is infinitely
  liquid for sells.
- The floor price $f$ is the guaranteed minimum redemption price for every
  m-token, backed by the reserve.

### Volatility = Slope

The slope of the active segment is the market's volatility (price sensitivity to
supply changes):

- Floor: slope $= 0$ → zero volatility, infinite sell-side liquidity.
- Ramp: slope $= m_1$ → high volatility, rapid price movement.
- Main: slope $= m_2$ → moderate volatility, long-run stability.

The ramp is steeper than main, so price moves faster through the ramp region.
This is the transition zone between the floor guarantee and the open market.

### Market Value and Guaranteed Value

Every m-token has two values: the **market price** $y(x)$ at the current supply,
and the **guaranteed value** $f$ (the floor). The difference between these two
is the token's exposure to price discovery.

The area under the curve can be split into two regions:

- **Guaranteed region** — the rectangle from $0$ to supply at height $f$. This
  is $f \cdot x$: the total value that is backed unconditionally. Every m-token
  can be redeemed for at least $f$, no matter what happens to supply. This value
  does not depend on other participants' behavior.
- **Discovery region** — the area above $f$, between $x_1$ and supply. This is
  the "pocket" where all price volatility lives. It is the integral of
  $y(s) - f$ over the ramp and main segments. This value exists only because
  supply is above $x_1$ — it is created by demand and destroyed by sells.

The market price of an m-token is $y(x) = f + \text{premium}$, where the premium
is the height of the curve above the floor at the current supply. The premium is
entirely a function of supply — more buyers push supply up the curve, increasing
the premium; sells push it back down. In the floor region, the premium is zero.

This decomposition clarifies what floor raising actually does: it converts
discovery-region area into guaranteed-region area. The total reserve doesn't
change, but more of it becomes unconditional. After a floor raise, the floor is
higher, the pocket of volatile area is smaller, and each token's guaranteed
value has increased. The market is the same size — it's just more of it is
locked in.

## Summary

The price curve is three lines defined by two immutable slopes ($m_1$, $m_2$)
and three state variables ($f$, $b_1$, $b_2$). The intersection points $x_1$ and
$x_2$ are derived from these. At launch $b_1 = b_2 = f$ and the curve is a
single ray $y = m_2\, x + f$. Floor raises separate the y-intercepts, opening up
the ramp and floor. Sell contractions compress them. The curve is static between
these events. From $(f, b_1, b_2, m_1, m_2)$, every price, every fill, and every
reserve balance is deterministically computable.

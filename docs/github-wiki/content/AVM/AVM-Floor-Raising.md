<!--
title: AVM Floor Raising
summary:
  How the area-preserving floor raise works — the ramp constraint creates an
  area deficit that funds a higher floor, preserving the reserve and spot price
  while converting volatile area into guaranteed area.
tags:
  - avm
  - floor
-->

Floor raising is the mechanism that evolves the
[price curve](AVM-Specification) over time. It increases the floor price $f$,
advances the ramp–main boundary $x_2$, and widens the ramp — all without
changing the reserve or the spot price. The result: more of the reserve becomes
unconditional, and every token's guaranteed value rises.

The floor can also rise through
[floor investment](Floor-Investment), which uses external capital
rather than rearranging the existing curve.

---

## What changes

- $f$ increases — the floor line rises.
- The ramp adjusts to connect the higher floor to the unchanged main schedule.
- $x_1$ and $x_2$ advance — the ramp widens.
- The main schedule $M(x)$ does not move.

At launch, the ramp has zero width ($x_1 = x_2$). Floor raises are what create
and widen the ramp. Each raise pushes the ramp–main boundary further along the
main schedule, opening up a larger transition zone behind the current supply.

---

## Invariants preserved

- **Reserve** — the total area under the curve from $0$ to supply is unchanged.
- **Spot price** — the price at the current supply does not move. Only the floor
  moves up.
- **Main schedule** — $M(x)$ is entirely untouched. The raise operates only on
  the floor and ramp.

---

## Why it works

The [ramp constraint](AVM-Specification) is the structural property that makes
this possible:

$$
R(x) < M(x) \quad \text{for all } x \in (x_1, x_2)
$$

When $x_2$ advances, a strip of the curve that was priced on the main schedule
is replaced by ramp. Because the ramp sits below the main in that strip, the
area under the curve **decreases**. This decrease compensates for the
**increase** in area from the taller floor rectangle ($\Delta f \cdot x_1$
grows). The two changes cancel exactly, and the reserve is unchanged.

In other words: the ramp traces a lower path than the main would through the
transition region. The area it _doesn't_ use — the deficit between ramp and main
— is the budget available for raising the floor. Every floor raise spends some
of this budget.

---

## Precondition

Supply must be above $x_2$ — the current price must be on the main schedule. If
supply is in the ramp or floor region, there is no room for $x_2$ to advance,
and no area deficit to redistribute.

---

## Magnitude

The new $x_2$ cannot exceed the current supply. If it did, the spot price would
change, violating the invariant. The further supply sits beyond $x_2$, the more
room there is to raise the floor. If the gap is large, the floor can increase
substantially in a single raise — the only limit is the area-preservation
constraint.

---

## What floor raising does to the value decomposition

Every token has a market price $y(x)$ and a guaranteed value $f$. The area under
the curve splits into a **guaranteed region** (the rectangle at height $f$) and
a **discovery region** (the area above $f$). Floor raising converts
discovery-region area into guaranteed-region area. The total reserve is
unchanged, but more of it is unconditional. After a raise, the volatile pocket
is smaller and each token's guaranteed minimum has increased.

This is the core promise of the AVM: participation in price discovery does not
come at the cost of permanent risk. Over time, the floor rises to capture gains,
and capital that was once exposed to volatility becomes permanently secured.

---

## The sell–raise cycle

Contraction and floor raising interact in an important way. Selling into the
ramp or floor shifts the main schedule down (see [Evolution](AVM-Evolution)),
which pulls $x_2$ back. This **lowers the bar** for future floor raises: once
$x_2$ has been pushed back, any subsequent buy that puts supply above $x_2$
re-enables floor raising — potentially at a lower price level than was
previously possible.

The system is self-healing. Sell-offs create the conditions for floor
appreciation on recovery. In the extreme case where supply is sold all the way
back to zero, the very next buy creates supply beyond $x_2$, enabling a floor
raise immediately.

---

## Contrast with floor investment

The area-preserving raise rearranges existing reserve: the ramp eats into
main-schedule territory to compensate for the taller floor.
[Floor investment](Floor-Investment) instead permits required
reserve to increase when existing surplus covers the candidate. Its geometry may
shorten or collapse the ramp and may consume the old $x_2$; the action still
requires the price at live supply to remain unchanged.

The two mechanisms compose. An operator can donate capital (creating surplus),
raise the floor from surplus, and then — if supply is beyond $x_2$ — perform an
additional area-preserving raise on top.

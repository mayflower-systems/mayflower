<!--
title: Floor Investment
summary:
  Surplus-funded floor raises that preserve the live price while allowing the
  floor/ramp geometry to consume existing boundaries.
tags:
  - avm-features
  - floor
-->

Floor investment is a mechanism for raising the floor using reserve surplus,
distinct from the area-preserving [floor raise](AVM-Floor-Raising). Its
defining property is the source of backing: the increase in required reserve is
paid by surplus rather than by an equal decrease elsewhere under the schedule.

---

## Surplus

The reserve equals the area under the [price curve](AVM-Specification) from
$0$ to supply. This is the minimum capital required for solvency — every token
must be redeemable at the price the curve assigns.

If the reserve holds more than this minimum, the difference is **surplus**:

$$
\text{surplus} = R_{\text{actual}} - \int_0^x y(s)\, ds
$$

Surplus can arise from external donations. A market operator (or anyone) can
deposit capital into the reserve without buying tokens. This capital goes
directly toward increasing the guaranteed value of every token in circulation.

---

## Accepted reshape

- $f$ increases — the floor rises.
- $x_1$ advances.
- The ramp may shorten or collapse to zero width.
- $x_2$ may remain fixed or advance, including past the old $x_2$.

The geometry layer only requires a valid continuous segmented schedule. It does
not attach economic meaning to crossing an old boundary.

The surplus-funded action applies the economic constraints at live supply $x$:

$$
f_{\text{new}} \le y_{\text{old}}(x)
$$

and

$$
y_{\text{new}}(x) = y_{\text{old}}(x)
$$

up to the explicitly selected numerical tolerance. The new required-reserve
upper bound must also be covered by effective reserve. A proposal that crosses
the old $x_2$ is therefore legal; a proposal that changes the live price is not.

---

## Why the spot price doesn't change

Live price preservation is checked directly at current supply. It is not
inferred from the position of $x_2$. This remains correct when supply is in any
segment and when the reshape consumes the old ramp–main boundary.

## Surplus absorbed

At current supply $x$, the backing consumed by an accepted reshape is the
increase in required reserve:

$$
\Delta R = \int_0^x y_{\text{new}}(s)\,ds - \int_0^x y_{\text{old}}(s)\,ds
$$

The surplus action requires $\Delta R \ge 0$ and requires the resulting total
area to remain covered. A market's operating rules determine how effective
reserve is calculated and what authorization or rate limits apply.

---

## Contrast with area-preserving raise

|                   | Area-preserving raise               | Floor investment                         |
| ----------------- | ----------------------------------- | ---------------------------------------- |
| Source of backing | area rearranged under the schedule  | existing reserve surplus                 |
| $f$               | increases                           | increases                                |
| Ramp              | adjusts                             | may shorten or collapse                  |
| $x_2$             | may advance                         | may stay fixed or advance past old $x_2$ |
| Required reserve  | preserved within selected tolerance | may increase, but must remain covered    |
| Live price        | preserved within selected tolerance | preserved within selected tolerance      |

The two mechanisms compose because they differ in acceptance policy rather than
in the schedule representation.

---

## Why it matters

Floor investment gives market operators a tool to directly strengthen the floor
without depending on organic buy volume pushing supply far beyond $x_2$. It
turns external capital into guaranteed value for every token holder.

An operator donating to the reserve is making a verifiable commitment: the floor
should be higher. Unlike spending capital on marketing or liquidity incentives,
the capital goes permanently into the reserve. It cannot be withdrawn. It backs
real tokens at a higher guaranteed price. Every unit donated translates directly
into a higher floor for the entire supply.

It also means the floor can rise even during periods of declining supply. The
area-preserving raise requires supply $> x_2$. Floor investment only requires
surplus. If a market is contracting but the operator believes in its long-term
value, donating capital and raising the floor signals conviction in a way that
is cryptographically verifiable and permanent.

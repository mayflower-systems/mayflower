<!--
title: Floor Investment
summary:
  Surplus-funded floor raises that preserve the main schedule and verify
  the reserve budget for proposed floor and ramp geometry.
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

Effective reserve includes held reserve and outstanding collateralized advances.
If this backing exceeds the required area, the difference is **surplus**:

$$
\text{surplus} = R_{\text{held}} + R_{\text{advanced}} - \int_0^x y(s)\, ds
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

The main schedule is preserved exactly. The proposal supplies a positive floor
increment, ramp end, and ramp width; it cannot change main placement. The ramp
is anchored to main at its proposed end. The floor join is checked separately,
allowing only an explicitly bounded upward gap for fixed-point rounding.

The new ramp end must be at or below characteristic progress. Therefore the
live price remains on the unchanged main schedule. Crossing the old ramp end
is legal; placing the new end beyond progress is not.

---

## Why the spot price doesn't change

Main evaluation depends on the immutable characteristic and a separately stored
vertical offset. Floor raising preserves that offset, so it cannot change any
main price. This is a property of the state transition, not a tolerated
live-price difference.

## Surplus absorbed

At current supply $x$, the backing consumed by an accepted reshape is the
increase in required reserve:

$$
\Delta R = \int_0^x y_{\text{new}}(s)\,ds - \int_0^x y_{\text{old}}(s)\,ds
$$

The surplus action requires the full computed enclosure to satisfy
$0 \le \Delta R \le G$, where $G$ is the caller's absolute reserve budget.
The proposed backing upper bound must also be covered by held reserve plus
outstanding collateralized advances. Authorization belongs to the host program.

Only the area through the new ramp end can change. When floor issuance or
redemption has made circulation differ from characteristic progress, the
comparison must include the resulting physical floor tail. On the progress
axis, add $(T-P)\Delta f$ to the change in the schedule's prefix integral.

---

## Contrast with area-preserving raise

|                   | Area-preserving raise               | Floor investment                         |
| ----------------- | ----------------------------------- | ---------------------------------------- |
| Source of backing | area rearranged under the schedule  | existing reserve surplus                 |
| $f$               | increases                           | increases                                |
| Ramp              | adjusts                             | may shorten or collapse                  |
| $x_2$             | may advance                         | may stay fixed or advance past old $x_2$ |
| Required reserve  | may shrink by an absolute tolerance  | may grow by an absolute funded budget   |
| Main schedule     | unchanged                           | unchanged                               |

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
area-preserving raise generally needs room to advance $x_2$. Floor investment
can use a fixed-$x_2$ reshape when surplus and the existing main price permit it. If a market is contracting but the operator believes in its long-term
value, donating capital and raising the floor signals conviction in a way that
is cryptographically verifiable and permanent.

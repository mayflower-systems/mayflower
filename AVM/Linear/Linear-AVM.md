<!--
title: Linear AVM
summary:
  Entry point for the piecewise-linear implementation of the AVM — three line
  segments (floor, ramp, main) defined by two immutable slopes and three state
  variables, with closed-form expressions for every operation.
tags:
  - linear-avm
  - math
-->

The piecewise-linear implementation of the [Amplifying Value Machine](Amplifying-Value-Machine). The
ramp and main are instantiated as linear functions, which gives closed-form
expressions for every intersection point, contraction, floor raise, fill price,
and reserve balance. The five parameters $(f, b_1, b_2, m_1, m_2)$ fully
determine the curve.

---

## Notes

- [Specification](Linear-AVM-Specification) — The formal definition: three
  lines, intersection points, initial conditions, reserve invariant, buying,
  selling, contraction rule, floor raising, and the market value / guaranteed
  value decomposition.
- [Evolution](Linear-AVM-Evolution) — How the curve mutates. Closed-form
  state-variable updates for every case (buy, sell, contraction, floor raise,
  floor investment, option execution) with change tables and the sell–raise
  cycle.
- [Floor Raising](Linear-AVM-Floor-Raising) — The area-preserving floor raise:
  closed-form expressions for how $f$, $b_1$, $x_1$, and $x_2$ change, the
  maximum floor formula, and a worked example.
- [Floor Investment](Linear-AVM-Floor-Investment) — Surplus-funded floor
  raises: state-variable changes, maximum floor constraint, surplus absorption
  formula, and a worked example.
- [Corollaries](Linear-AVM-Corollaries) — Derived geometric properties: slope
  from cost-to-double, the square root law, why the ramp matters, maximum floor
  raise bounds, and price impact per segment.

---

## Key Concepts

- **Five parameters** — Two immutable slopes ($m_1$, $m_2$) and three state
  variables ($f$, $b_1$, $b_2$). The intersection points $x_1$ and $x_2$ are
  derived. From these five numbers, every price, fill, and reserve balance is
  deterministically computable.
- **Closed-form everything** — Linearity means every integral is a trapezoid.
  Fill prices, reserve balances, contraction updates, and floor-raise solutions
  are algebraic — no numerical integration required.
- **Square root law** — Price grows with the square root of capital. A 10x
  return costs 33x more capital than a 2x return. This is a consequence of the
  linear structure.

---

## How it connects

- MayflowerOS uses this piecewise-linear AVM. The general [AVM](Amplifying-Value-Machine)
  specification defines the constraints on the curve; this note family gives the
  equations for the linear case.
- The two slopes are immutable curve parameters. Floor evolution follows the
  [geometric constraints](Linear-AVM-Floor-Raising) and the host's
  floor-raising policy; [channel weights](Emission-Channel-Weighting)
  control emission allocation.
- The optional [AVM features](AVM-Features) (call options, floor investment,
  cash advance) are defined in implementation-agnostic terms but have
  linear-specific equations here (see [Evolution](Linear-AVM-Evolution) and
  [Floor Investment](Linear-AVM-Floor-Investment)).

<!--
title: Linear AVM Floor Investment
summary:
  Linear implementation of surplus-funded floor raises with unchanged main
  placement and reserve-backed ramp reshaping.
tags:
  - linear-avm
  - floor
  - math
-->

The linear implementation of
[floor investment](Floor-Investment). Surplus funding does not
prescribe one unique reshape. It permits any valid linear segmented candidate
that preserves the entire main schedule and whose required area is covered by
effective reserve.

---

## State-variable changes

Floor investment raises $f$ while preserving the main intercept $b_2$:

| Variable | Constraint |
| -------- | ---------- |
| $f$ | **increases** by a positive increment |
| $b_1$ | derived by anchoring the ramp at the proposed $x_2$ |
| $b_2$ | unchanged |

The slopes $m_1$ and $m_2$ are immutable. Continuity at the proposed ramp end
requires:

$$
b_1^{\text{new}} = b_2 - (m_1-m_2)x_2^{\text{new}}
$$

The supplied ramp width determines $x_1^{\text{new}}$. Its floor join is checked
against:

$$
f_{\text{new}} = m_1 x_1^{\text{new}} + b_1^{\text{new}}
$$

with only a bounded upward gap permitted in fixed-point arithmetic. The ramp
may shorten or collapse, and $x_2$ may stay fixed or advance. Both boundaries
must remain at or behind current characteristic progress.

## Main-price preservation

Every main price remains $m_2 x + b_2$, independent of the proposed floor and
ramp. Since the new ramp end cannot exceed progress, the current price remains
on this unchanged main. No main-price tolerance is needed.

---

## Surplus absorption

The surplus required is the increase in total area at live supply:

$$
\Delta R = \int_0^x y_{\text{new}}(s)\,ds - \int_0^x y_{\text{old}}(s)\,ds
$$

For the fixed-$x_2$, fixed-intercept special case below, the floor replaces a
portion of the old ramp. The area change simplifies to:

$$
\Delta R = \frac{(f_{\text{new}}-b_1)^2-(f-b_1)^2}{2m_1}
$$

This is the amount of surplus consumed. If the available surplus is less than
$\Delta R$, the floor can only be raised to whatever $f_{\text{new}}$ the
surplus can support.

---

## Worked example

**Setup:** $m_1 = 0.0003$, $m_2 = 0.00015$, $f = 0.10$, $b_1 = 0.04$,
$b_2 = 0.10$. Supply $= 600$.

**Before the raise:**

$$
x_1 = \frac{0.10 - 0.04}{0.0003} = 200, \qquad x_2 = \frac{0.10 - 0.04}{0.0003 - 0.00015} = 400
$$

Price at $x_2$: $y(400) = 0.0003 \times 400 + 0.04 = 0.16$. This is the maximum
floor for the fixed-$x_2$ special case.

**Raise floor to $f_{\text{new}} = 0.12$ using the fixed-$x_2$ special case:**

New $x_1$:

$$
x_1^{\,\text{new}} = \frac{0.12 - 0.04}{0.0003} = 266.\overline{6}
$$

$x_2$ unchanged at $400$. Spot price at supply $= 600$:
$y(600) = 0.00015 \times 600 + 0.10 = 0.19$. Unchanged.

Surplus consumed:

$$
\Delta R = \frac{0.08^2-0.06^2}{2 \times 0.0003}
         = \frac{0.0028}{0.0006} = \frac{14}{3}
$$

The floor rose 20% (from $0.10$ to $0.12$), the floor region widened from $200$
to $266.\overline{6}$, and $4.\overline{6}$ units of capital were absorbed from surplus.
Other legal proposals may move or collapse the ramp while satisfying the same
main-preservation and backing rules.

---

## Contrast with the area-preserving raise

|                   | Area-preserving raise              | Floor investment                     |
| ----------------- | ---------------------------------- | ------------------------------------ |
| Source of backing | area rearranged under the schedule | reserve surplus                      |
| $f$               | increases                          | increases                            |
| Ramp              | adjusts                            | may shorten or collapse              |
| $x_2$             | may advance                        | may stay fixed or advance            |
| Required reserve  | preserved within tolerance         | may increase but must remain covered |
| Live price        | preserved                          | preserved                            |

The two mechanisms
[compose](Floor-Investment#contrast-with-area-preserving-raise)
because they use the same schedule representation with different action-level
reserve policy.

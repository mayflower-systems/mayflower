<!--
title: Linear AVM Floor Investment
summary:
  Linear implementation of surplus-funded floor raises with live-price
  preservation and reserve-backed ramp reshaping.
tags:
  - linear-avm
  - floor
  - math
-->

The linear implementation of
[floor investment](Floor-Investment). Surplus funding does not
prescribe one unique reshape. It permits any valid linear segmented candidate
that preserves the live price and whose required area is covered by effective
reserve.

---

## State-variable changes

Floor investment always raises $f$. The ramp intercept and main intercept depend
on the selected candidate geometry:

| Variable | Constraint                                     |
| -------- | ---------------------------------------------- |
| $f$      | **increases** to $f_{\text{new}}$              |
| $b_1$    | derived from the proposed $x_1$                |
| $b_2$    | derived from the proposed $x_2$ and continuity |

The slopes $m_1$ and $m_2$ are immutable.

For fixed slopes $m_1$ and $m_2$:

$$
b_1^{\text{new}} = f_{\text{new}} - m_1 x_1^{\text{new}}
$$

and continuity at the proposed ramp end requires:

$$
b_2^{\text{new}} = (m_1-m_2)x_2^{\text{new}} + b_1^{\text{new}}
$$

The special case $x_2^{\text{new}}=x_2^{\text{old}}$ can leave both line
intercepts unchanged, but this is not required. The ramp may also collapse with
$x_1^{\text{new}}=x_2^{\text{new}}$, including at a point beyond the old $x_2$.

## Live-price constraint

At current supply $x$, the semantic action requires:

$$
f_{\text{new}} \le y_{\text{old}}(x), \qquad
y_{\text{new}}(x)=y_{\text{old}}(x)
$$

This direct check replaces the old restriction
$f_{\text{new}}\le y(x_2^{\text{old}})$. Crossing the old boundary is
geometrically legal; changing the price at live supply is not.

---

## Surplus absorption

The surplus required is the increase in total area at live supply:

$$
\Delta R = \int_0^x y_{\text{new}}(s)\,ds - \int_0^x y_{\text{old}}(s)\,ds
$$

For the fixed-$x_2$, fixed-intercept special case below, only the floor portion
changes and this simplifies to:

$$
\Delta R = \frac{1}{m_1}\!\left[\,f_{\text{new}}\,(f_{\text{new}} - b_1) - f\,(f - b_1)\,\right]
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
\Delta R = \frac{1}{0.0003}\!\left[\,0.12 \times 0.08 - 0.10 \times 0.06\,\right] = \frac{0.0096 - 0.006}{0.0003} = 12
$$

The floor rose 20% (from $0.10$ to $0.12$), the floor region widened from $200$
to $266.\overline{6}$, and $12$ units of capital were absorbed from surplus.
Other legal proposals may move or collapse the ramp while satisfying the same
live-price and backing rules.

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

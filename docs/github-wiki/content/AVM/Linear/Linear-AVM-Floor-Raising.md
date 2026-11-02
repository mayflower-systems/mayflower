<!--
title: Linear AVM Floor Raising
summary:
  Linear implementation of the area-preserving floor raise — closed-form
  expressions for how f, b₁, x₁, and x₂ change, the maximum floor formula, and a
  worked numeric example.
tags:
  - linear-avm
  - floor
  - math
-->

The linear implementation of the
[area-preserving floor raise](AVM-Floor-Raising). This note gives the
closed-form equations for how the [state variables](Linear-AVM-Specification)
change when the floor is raised, the formula for the maximum achievable floor,
and a worked example.

---

## State-variable changes

A floor raise modifies two of the three state variables:

| Variable | Change                                         |
| -------- | ---------------------------------------------- |
| $f$      | **increases** to $f_{\text{new}}$              |
| $b_1$    | **decreases** (derived from area preservation) |
| $b_2$    | unchanged                                      |

The slopes $m_1$ and $m_2$ are immutable.

### Derived intersection points

Both advance as a consequence:

$$
x_1 = \frac{f - b_1}{m_1}, \qquad x_2 = \frac{b_2 - b_1}{m_1 - m_2}
$$

Since $f$ increased and $b_1$ decreased (with $b_2$ fixed), both $x_1$ and $x_2$
move right. The ramp width $(x_2 - x_1)$ increases.

### Computing $b_1^{\,\text{new}}$

The new $b_1$ is determined by the area-preservation constraint: the total area
under the curve from $0$ to supply must be unchanged. Given the new floor
$f_{\text{new}}$, solve for $b_1^{\,\text{new}}$ such that:

$$
\int_0^x y_{\text{new}}(s)\, ds = \int_0^x y_{\text{old}}(s)\, ds
$$

Because both curves are piecewise-linear and share the same main line (same
$b_2$, $m_2$), the integrals reduce to algebraic expressions in $f_{\text{new}}$
and $b_1^{\,\text{new}}$. The result is a closed-form solution for
$b_1^{\,\text{new}}$ in terms of $f_{\text{new}}$, $b_2$, $m_1$, $m_2$, and the
current supply.

---

## Maximum floor

The new $x_2$ cannot exceed the current supply (otherwise the spot price would
change). This bounds the maximum achievable floor.

For a market sitting entirely on the main segment (supply well above $x_2$), the
approximate maximum floor as a fraction of the current price is:

$$
\frac{f_{\max}}{y(\text{supply})} \approx 1 - \sqrt{\frac{m_2}{m_1}}
$$

If $m_1 = 2\, m_2$: $f_{\max}/y \approx 1 - 1/\sqrt{2} \approx 29\%$ of the
current price can be locked in as floor in a single raise (with enough room).
The actual limit depends on how far supply exceeds $x_2$ and the current floor
level.

---

## Worked example

**Setup:** $m_1 = 0.0003$, $m_2 = 0.00015$, $f = 0.10$, $b_1 = 0.04$,
$b_2 = 0.10$. Supply $= 600$.

**Before the raise:**

$$
x_1 = \frac{0.10 - 0.04}{0.0003} = 200, \qquad x_2 = \frac{0.10 - 0.04}{0.0003 - 0.00015} = 400
$$

Supply (600) is above $x_2$ (400), so a floor raise is possible. Spot price:
$y(600) = 0.00015 \times 600 + 0.10 = 0.19$.

**Raise floor to $f_{\text{new}} = 0.12$:**

$b_1$ decreases to satisfy area preservation. The new intersection points
advance:

$$
x_2^{\,\text{new}} < 600 \quad \checkmark
$$

The floor rose from $0.10$ to $0.12$ — a 20% increase — while the spot price at
supply $= 600$ stays at $0.19$. The reserve is unchanged. The guaranteed value
per token increased from $0.10$ to $0.12$, and the ramp widened behind the
current supply.

---

## Precondition

Supply must be in the main region: $\text{supply} > x_2$. Equivalently, the spot
price must be on the main line.

If supply is in the ramp ($x_1 \le \text{supply} < x_2$) or the floor
($\text{supply} < x_1$), there is no room for $x_2$ to advance and no area
deficit to redistribute. The floor cannot be raised by area-preservation in
these states — only [floor investment](Floor-Investment)
(surplus-funded) can raise the floor when supply is at or below $x_2$.

---
title: Evolution (Linear)
summary:
  Linear implementation of every curve mutation — closed-form expressions for
  how f, b₁, and b₂ change during buys, sells, contraction, floor raises, floor
  investment, and option execution. Each case gives the exact state-variable
  update and the resulting x₂ motion.
tags:
  - linear-avm
  - math
---

The linear implementation of [[avm--evolution|AVM evolution]]. This note gives
the closed-form state-variable updates for every curve mutation in the
[[linear-avm--specification|piecewise-linear system]]. The general note
describes _what_ changes in each case; this note gives the _equations_.

## The three lines

| Line  | Equation            | Y-intercept |
| ----- | ------------------- | ----------- |
| Floor | $y = f$             | $f$         |
| Ramp  | $y = m_1\, x + b_1$ | $b_1$       |
| Main  | $y = m_2\, x + b_2$ | $b_2$       |

The slopes $m_1$ (ramp) and $m_2$ (main) are immutable. The state variables are
the three y-intercepts: $f$, $b_1$, $b_2$.

The intersection points are derived:

$$
x_1 = \frac{f - b_1}{m_1}, \quad x_2 = \frac{b_2 - b_1}{m_1 - m_2}
$$

---

## Buy

Supply increases. No state variable changes. See
[[avm--evolution#buy|general treatment]].

| $f$ | $b_1$ | $b_2$ |
| --- | ----- | ----- |
| —   | —     | —     |

---

## Sell (supply stays in main)

Supply decreases but remains $\ge x_2$. No state variable changes. See
[[avm--evolution#sell-supply-stays-on-main|general treatment]].

| $f$ | $b_1$ | $b_2$ |
| --- | ----- | ----- |
| —   | —     | —     |

---

## Sell into ramp ($x_1 \le x' < x_2$)

Supply drops below $x_2$ but stays at or above $x_1$. The ramp line does not
move. The main line re-anchors through the ramp at the new supply:

$$
b_2^{\,\text{new}} = (m_1 - m_2)\, x' + b_1
$$

Since $b_1$ is unchanged, $x_1$ does not move. $b_2$ dropped, so $x_2$ contracts
to the new supply.

| $f$ | $b_1$ | $b_2$         |
| --- | ----- | ------------- |
| —   | —     | **decreases** |

The main line is permanently lower. Subsequent buys walk up the shifted-down
main. Only a floor raise can advance $x_2$ again. See
[[avm--evolution#sell-into-ramp|general treatment]].

---

## Sell into floor ($x' < x_1$)

Supply drops below $x_1$. The ramp collapses. Both lines re-anchor through
$(x', f)$:

$$
b_1^{\,\text{new}} = f - m_1\, x', \quad b_2^{\,\text{new}} = f - m_2\, x'
$$

Now $x_1 = x_2 = x'$ — the ramp has zero width. At the limit $x' = 0$:
$b_1 = b_2 = f$, and the curve is a single ray $y = m_2\, x + f$ — the launch
shape (though $f$ may have been raised). See
[[avm--evolution#sell-into-floor|general treatment]].

| $f$ | $b_1$      | $b_2$         |
| --- | ---------- | ------------- |
| —   | **resets** | **decreases** |

---

## Floor raise (area-preserving)

$f$ increases. $b_2$ is unchanged. $b_1$ **decreases** so that the ramp line, at
its steeper slope, connects the higher floor to the unchanged main line. See
[[linear-avm--floor-raising]] for the closed-form derivation of
$b_1^{\,\text{new}}$ and the maximum floor formula.

Since $b_2$ is fixed and $b_1$ decreased: $x_2 = (b_2 - b_1)/(m_1 - m_2)$
increases. Since $f$ increased and $b_1$ decreased: $x_1 = (f - b_1)/m_1$
increases. The ramp widens.

| $f$           | $b_1$         | $b_2$ |
| ------------- | ------------- | ----- |
| **increases** | **decreases** | —     |

**Precondition:** supply $> x_2$. The new $x_2$ cannot exceed supply.

**Invariants preserved:** reserve (area under curve from $0$ to supply), price
at supply, $b_2$, $m_1$, $m_2$. See
[[avm--evolution#floor-raise-area-preserving|general treatment]].

---

## Floor investment (surplus-funded)

The floor rises and the selected geometry may change $b_1$, $b_2$, $x_1$, and
$x_2$. The ramp may shorten or collapse, including beyond the old $x_2$. See
[[linear-avm--floor-investment]] for the linear constraints and
[[avm-features--floor-investment]] for the general policy.

| $f$           | Ramp                | $x_2$            |
| ------------- | ------------------- | ---------------- |
| **increases** | shortens/can vanish | fixed or forward |

**Precondition:** $f_{\text{new}}$ does not exceed the old live price, the
candidate preserves the live price, and effective reserve covers the new
required area.

**Contrast with area-preserving raise:** both may reshape the ramp. The
surplus-funded action permits a covered area increase; the area-preserving
action does not.

---

## Option execution

A [[avm-features--call-options|perpetual call option]] is executed. The buyer
pays $f$ per token, and $n$ new tokens are minted. Both lines shift right by
$n$:

$$
b_1 \to b_1 - m_1\, n, \quad b_2 \to b_2 - m_2\, n
$$

$f$ does not change. Since both y-intercepts decreased by amounts proportional
to their slopes, both intersection points shift right by exactly $n$ — a pure
rightward translation.

Supply increases from $x$ to $x + n$. Price is unchanged:
$y_{\text{new}}(x + n) = y_{\text{old}}(x)$.

Reserve invariant holds: the buyer paid $f \cdot n$, covering the $n$ new units
of floor region. See [[avm--evolution#option-execution|general treatment]].

| $f$ | $b_1$                    | $b_2$                    |
| --- | ------------------------ | ------------------------ |
| —   | **decreases** by $m_1 n$ | **decreases** by $m_2 n$ |

Unlike a floor raise (which preserves $b_2$), option execution lowers both
y-intercepts. Unlike sell contraction (which lowers $b_2$ only), option
execution shifts both lines in the same direction by the same amount — a
translation, not a contraction.

---

## How $x_2$ moves

$x_2 = (b_2 - b_1)/(m_1 - m_2)$. It shifts whenever $b_1$ or $b_2$ changes:

| Event                         | $b_1$         | $b_2$         | $x_2$ direction | Which line moves?              |
| ----------------------------- | ------------- | ------------- | --------------- | ------------------------------ |
| Floor raise (area-preserving) | **decreases** | —             | **forward**     | ramp adjusts, main stays       |
| Floor investment (surplus)    | —             | —             | —               | neither; only $f$ rises        |
| Sell contraction              | —             | **decreases** | **backward**    | main shifts down, ramp stays   |
| Option execution              | **decreases** | **decreases** | **forward**     | both shift right (translation) |

See [[avm--evolution#how-x_2-moves|general treatment]] for the
implementation-agnostic version.

---

## The sell–raise cycle

Selling into the ramp or floor lowers $b_2$, which pulls $x_2$ back. This lowers
the bar for future floor raises:

1. Supply is at 100, $x_2$ is at 80. Floor can be raised (supply $> x_2$).
2. A large sell pushes supply to 50. $b_2$ drops, pulling $x_2$ to 50.
3. New buys push supply to 60. Now supply $> x_2$ again — a floor raise is
   possible.
4. The floor raise increases $f$ and decreases $b_1$, sliding $x_2$ forward
   along the (now lower) main line.

The system is self-healing: sell-offs create room for floor appreciation on
recovery. See [[avm--evolution#the-sell–raise-cycle|general treatment]].

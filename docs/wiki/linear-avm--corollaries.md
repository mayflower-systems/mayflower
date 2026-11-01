---
title: Corollaries (Linear)
summary:
  Derived geometric properties of the piecewise-linear price curve — slope from
  cost-to-double, the square root law for capital vs. ROI, why the ramp matters
  for early adoption, maximum floor raise bounds, and price impact formulas per
  segment.
tags:
  - linear-avm
  - math
---

Useful geometric properties derived from the
[[linear-avm--specification|piecewise-linear price curve]]. These help with
parameter selection, modeling, and intuition. They depend on the linear
structure — none of them generalize to an arbitrary [[avm--specification|AVM]]
without modification.

---

## Slope from floor price and cost-to-double

On a linear segment with slope $m$ and starting price $f$, the cost to double
the price (from $f$ to $2f$) is:

$$
K = \frac{3\, f^2}{2\, m}
$$

Rearranging gives the slope as a function of $f$ and $K$:

$$
m = \frac{3\, f^2}{2\, K}
$$

This works for any magnitude of $f$. If $f$ is tiny (e.g., $10^{-18}$), $m$
becomes extremely small — but the relationships hold exactly.

**Example:** $f = 0.1$, $K = 100$. Then $m = 3 \cdot 0.01 / 200 = 0.00015$. The
supply needed to double the price is $\Delta x = f/m = 666.67$ tokens, and the
cost is the trapezoid: $(0.1 + 0.2)/2 \times 666.67 = 100 = K$. ✓

---

## The square root law

On a single linear segment starting at price $f$, the price multiplier (ROI)
after $C$ total capital inflow is:

$$
\text{ROI} = \sqrt{1 + \frac{3\,C}{K}}
$$

Price grows with the **square root** of capital. Doubling the capital does not
double the price — it barely moves. The curve gets "heavier" the higher the
price goes.

| Target ROI | Capital required ($C/K$) |
| :--------: | :----------------------: |
|     2x     |            1             |
|     5x     |            8             |
|    10x     |            33            |
|    50x     |           833            |
|    100x    |          3,333           |

Inverted — the capital required to hit a given ROI:

$$
C = \frac{K}{3} \cdot (\text{ROI}^2 - 1)
$$

Capital scales with the **square** of the return. A 10x return costs 33 times
more capital than a 2x return. A 100x return costs 3,333 times more.

---

## Why the ramp matters

On the main segment alone (slope $m_2$), the cost-to-double $K$ is large — by
design, for stability. Early capital would need enormous inflows to move the
price.

The ramp (slope $m_1 > m_2$) has a lower effective $K$:

$$
K_{\text{ramp}} = \frac{3\, f^2}{2\, m_1} < \frac{3\, f^2}{2\, m_2} = K_{\text{main}}
$$

If $m_1 = 2\, m_2$, the ramp's cost-to-double is half that of main. Early
capital moves the price faster through the ramp, allowing meaningful ROI
(10x–20x) with reasonable inflows, before the heavier main curve takes over for
long-run stability.

This is the geometric justification for the piecewise design: a single-slope
curve is either too heavy for early adoption or too light for stability. The
ramp solves both by giving early capital a steeper, cheaper path that
transitions into the stable main curve. See also the
[[avm--specification#the-ramp-constraint|ramp constraint]] for the general
structural property this exploits.

---

## Maximum floor raise

The floor can be raised as long as $\text{supply} > x_2$. The maximum new floor
is bounded by the area-preservation constraint: the increase in floor area
($\Delta f \cdot \text{supply}$) must equal the decrease in area from sliding
$x_2$ forward (replacing main-slope area with ramp-slope area).

For a market sitting entirely on the main segment (supply well above $x_2$), the
approximate maximum floor as a fraction of the current price is:

$$
\frac{f_{\max}}{y(\text{supply})} \approx 1 - \sqrt{\frac{m_2}{m_1}}
$$

If $m_1 = 2\, m_2$: $f_{\max}/y \approx 1 - 1/\sqrt{2} \approx 29\%$ of the
current price can be locked in as floor in a single raise (with enough room). In
practice, the actual limit depends on how far supply exceeds $x_2$ and the
current floor level. See [[linear-avm--floor-raising]] for the full derivation.

---

## Price impact on each segment

For a buy of $\Delta x$ tokens on a single segment with slope $m$:

$$
\Delta y = m \cdot \Delta x
$$

$$
\text{cost} = \Delta x \cdot \left(y_{\text{start}} + \frac{m \cdot \Delta x}{2}\right)
$$

$$
\bar{y} = y_{\text{start}} + \frac{m \cdot \Delta x}{2}
$$

The fill price is the midpoint between the starting and ending price. This is
exact for a linear segment — no approximation.

On the floor ($m = 0$): $\Delta y = 0$, fill price = $f$. No slippage.

On the ramp ($m = m_1$): price impact is $m_1 \cdot \Delta x$ per token of
supply change. This is the most volatile region.

On main ($m = m_2$): price impact is $m_2 \cdot \Delta x$. Smaller per unit of
supply, but this is where the market spends most of its life.

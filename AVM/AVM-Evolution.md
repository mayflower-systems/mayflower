<!--
title: AVM Evolution
summary:
  How the AVM price curve mutates — a complete walkthrough of every state change
  (buy, sell, sell into ramp, sell into floor, area-preserving floor raise,
  surplus-funded floor investment, option execution) with a summary table and
  the sell–raise cycle.
tags:
  - avm
  - pricing
-->

The [price curve](AVM-Specification) is static most of the time. Buys and
sells move supply along the existing curve without changing its shape. The curve
only mutates during specific events: a sell contraction, a
[floor raise](AVM-Floor-Raising), a
[floor investment](Floor-Investment), or an
[option execution](Call-Options). This note walks through every
case.

---

## Buy

Supply increases. No schedule moves. The buyer pays the integral under the
existing curve. The reserve grows by exactly the cost paid.

| Floor $f$ | Ramp | Main | $x_2$ direction |
| --------- | ---- | ---- | --------------- |
| —         | —    | —    | —               |

---

## Sell (supply stays on main)

Supply decreases but remains at or above $x_2$. No schedule moves. The seller
receives the integral under the curve. The reserve shrinks by the proceeds paid
out.

| Floor $f$ | Ramp | Main | $x_2$ direction |
| --------- | ---- | ---- | --------------- |
| —         | —    | —    | —               |

---

## Sell into ramp

Supply drops below $x_2$ but stays at or above $x_1$. The ramp is unchanged. The
main schedule shifts down so that it meets the ramp at the new, lower supply.
The ramp–main boundary contracts: $x_2$ moves back to the new supply.

| Floor $f$ | Ramp | Main        | $x_2$ direction |
| --------- | ---- | ----------- | --------------- |
| —         | —    | shifts down | backward        |

After this, the main schedule is permanently lower. Subsequent buys walk up the
shifted-down main. Only a floor raise can advance $x_2$ again.

---

## Sell into floor

Supply drops below $x_1$. The ramp collapses to zero width. Both the ramp and
main schedules re-anchor at the new supply point at height $f$, so
$x_1 = x_2 = x'$. If supply returns all the way to $0$, the curve resets to its
launch shape (though $f$ may have been raised by prior floor raises).

| Floor $f$ | Ramp       | Main        | $x_2$ direction |
| --------- | ---------- | ----------- | --------------- |
| —         | re-anchors | shifts down | backward        |

---

## Floor raise (area-preserving)

The floor rises. The main schedule does not move. The ramp adjusts to connect
the higher floor to the unchanged main. See [Floor Raising](AVM-Floor-Raising)
for the full treatment.

$f$ increases. The ramp adjusts so that, at its steeper path, it connects the
higher floor to the unchanged main schedule. Both $x_1$ and $x_2$ advance — the
ramp widens.

| Floor $f$ | Ramp    | Main | $x_2$ direction |
| --------- | ------- | ---- | --------------- |
| increases | adjusts | —    | forward         |

**Precondition:** supply $> x_2$. The new $x_2$ cannot exceed supply.

**Invariants preserved:** reserve (area under curve), spot price, the main
schedule $M(x)$.

---

## Floor investment (surplus-funded)

External capital is donated to the reserve, creating a surplus. The floor rises
to absorb the surplus. See [Floor Investment](Floor-Investment)
for the full treatment.

The floor rises and the ramp may shorten or collapse. The proposed $x_2$ may
remain fixed or advance past the old boundary. The action accepts the reshape
only when effective reserve covers the new required area and the price at live
supply is unchanged.

| Floor $f$ | Ramp                | $x_2$ direction  |
| --------- | ------------------- | ---------------- |
| increases | shortens/can vanish | fixed or forward |

**Precondition:** the new floor does not exceed the old price at live supply,
the candidate preserves that live price, and effective reserve covers its
required area.

**Contrast with area-preserving raise:** both operations may reshape the ramp.
The difference is whether the action requires area preservation or permits
surplus to fund an area increase.

---

## Option execution

A [perpetual call option](Call-Options) is executed. The buyer
pays $f$ per token and $n$ new tokens are minted. Both the ramp and main
schedules shift right by $n$. The floor $f$ does not change. Both $x_1$ and
$x_2$ advance by exactly $n$ — a pure rightward translation, widening the floor
region.

Supply increases from $x$ to $x + n$, but the spot price is unchanged:
$y_{\text{new}}(x + n) = y_{\text{old}}(x)$.

The reserve invariant holds because the buyer paid $f \cdot n$, which exactly
covers the $n$ new units of floor region.

| Floor $f$ | Ramp         | Main         | $x_2$ direction |
| --------- | ------------ | ------------ | --------------- |
| —         | shifts right | shifts right | forward         |

Unlike a floor raise (which preserves the main and adjusts the ramp), option
execution shifts both schedules together. And unlike sell contraction (which
shifts only the main down), option execution moves both schedules in the same
direction by the same amount.

---

## How $x_2$ moves

$x_2$ is the ramp–main boundary. It shifts whenever either schedule moves:

| Event                         | Ramp         | Main         | $x_2$ direction |
| ----------------------------- | ------------ | ------------ | --------------- |
| Floor raise (area-preserving) | adjusts      | —            | forward         |
| Floor investment (surplus)    | —            | —            | —               |
| Sell contraction              | —            | shifts down  | backward        |
| Option execution              | shifts right | shifts right | forward         |

Floor raising and sell contraction are complementary, not symmetric. Floor
raising adjusts the ramp to reach a higher floor while leaving the main in
place. Sell contraction shifts the main down while leaving the ramp in place.
Option execution moves both together. Floor investment is the only mutation that
moves $f$ alone — it changes the floor without touching either schedule.

---

## The sell–raise cycle

Sell contraction and floor raising interact in an important way. Selling into
the ramp or floor shifts the main schedule down, pulling $x_2$ back. This lowers
the bar for future floor raises: once $x_2$ has been pushed back, any subsequent
buy that puts supply above $x_2$ re-enables floor raising — potentially at a
lower price level than was previously possible.

This means sell pressure does not permanently damage the floor mechanism. It can
create the conditions for a higher floor afterward:

1. Supply is above $x_2$. Floor can be raised.
2. A large sell pushes supply down. The main schedule shifts down, pulling $x_2$
   back to the new supply.
3. New buys push supply up. Now supply is above $x_2$ again — a floor raise is
   possible.
4. The floor raise increases $f$ and adjusts the ramp, sliding $x_2$ forward
   along the (now lower) main schedule.

The system is self-healing: sell-offs create room for floor appreciation on
recovery.

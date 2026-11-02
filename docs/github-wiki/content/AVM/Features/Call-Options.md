<!--
title: Call Options
summary:
  Perpetual call options with strike price equal to the floor — an optional
  add-on to the AVM that mints new tokens at floor price, translates the curve
  rightward, and preserves the reserve invariant without changing the spot
  price.
tags:
  - avm-features
  - options
-->

The AVM can support a perpetual call option with strike price equal to the
current floor. This is not part of the core pricing mechanism — it is an add-on
that some markets may enable. Execution mints new tokens at the floor price,
widens the floor region, and leaves the spot price unchanged.

---

## Why the strike price is the floor

The floor is the minimum amount of reserved capital backing each token. It is
the guaranteed redemption value — the price at which the reserve will always buy
back a token. A new token can only be issued if at least $f$ of capital enters
the reserve to back it. The floor is therefore the minimum price at which the
AVM can issue a token without breaking the solvency invariant.

---

## Execution

The buyer pays $f$ per token and receives newly minted tokens. The payment goes
into the reserve.

If $n$ options are executed:

- The buyer pays $f \cdot n$. This capital enters the reserve.
- $n$ new tokens are minted.
- Both the ramp and main schedules shift right by $n$: $x_1 \to x_1 + n$,
  $x_2 \to x_2 + n$.
- The floor region widens by $n$ units.
- The floor $f$ does not change.

---

## Price is unchanged

Supply increases from $x$ to $x + n$, but the price at the new supply equals the
price at the old supply:

$$
y_{\text{new}}(x + n) = y_{\text{old}}(x)
$$

The curve shifted right by $n$, so the same price that was at position $x$ is
now at position $x + n$. No price impact.

---

## Reserve invariant holds

The reserve must equal the area under the curve from $0$ to supply. After the
rightward shift, the new curve has $n$ additional units of floor region at
height $f$. The rest of the curve is identical, just translated. The new
required reserve is $R_{\text{old}} + f \cdot n$, which is exactly what the
buyer paid.

---

## Arbitrage and the execution fee

If a buyer executes an option (paying $f$) and immediately redeems the new
token, they receive approximately $y(x)$ — the market price. The profit is
$y(x) - f$: the premium above the floor. This is the value of the
[discovery region](AVM-Specification) captured per token.

This arbitrage is priced by the **execution fee**, set through the market's
authorized fee configuration. Availability of option operations depends on the
application; Mayflower Reserve does not enable them. The fee makes execution
profitable only when the premium is large enough to cover it. In practice,
options are rarely executed — they only make sense when the market price is
substantially above the floor.

---

## Effect on the curve

Unlike a floor raise (which preserves the main schedule and adjusts the ramp),
option execution shifts both schedules in the same direction by the same amount
— a pure rightward translation. And unlike sell contraction (which shifts the
main schedule down), option execution moves both schedules together without
changing their relationship. The ramp–main boundary $x_2$ advances by exactly
$n$.

| $f$ | Ramp         | Main         | $x_2$ direction |
| --- | ------------ | ------------ | --------------- |
| —   | shifts right | shifts right | forward         |

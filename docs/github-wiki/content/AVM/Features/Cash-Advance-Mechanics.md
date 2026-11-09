<!--
title: Cash Advance Mechanics
summary:
  Capacity, reserve movement, collateral, fees, and settlement of AVA advances.
tags:
  - avm-features
  - cash-advance
-->

The [cash advance](Cash-Advance-Overview) uses the AVM's reserve asset and the
floor backing of a position. Equations below use exact normalized quantities;
rounding rules are stated separately.

## Position and capacity

Let $`n\ge0`$ be the position's AVA quantity, $`d\ge0`$ its outstanding reserve
advance, and $`F\ge0`$ the current floor price. A valid position satisfies

```math
d\le Fn.
```

In fixed-point arithmetic, capacity is $`\lfloor Fn\rfloor_\delta`$, rounding
down to the accounting grid $`\delta`$. If $`F=0`$, capacity is zero. All
accepted changes must leave the target position within capacity.

For $`F\gt 0`$, the ideal minimum collateral for principal $`d`$ is $`d/F`$. When
AVA units are discrete, the quantity must be rounded up sufficiently to pass the
capacity check. The application must prevent transfers or redemptions that would
leave less collateral than required.

If the floor rises from $`F`$ to $`(1+r)F`$, the ideal collateral requirement
changes from $`d/F`$ to $`d/((1+r)F)`$: a reduction by the fraction $`r/(1+r)`$,
not by $`r`$. For example, a 25% floor increase reduces that requirement by 20%.
Actual withdrawal or transfer still requires an authorized action.

## Reserve movement

Let $`H`$ be liquid reserve and $`D`$ total outstanding advances. For a target
principal $`d'`$, define $`\Delta d=d'-d`$. With no simultaneous trade:

```math
H'=H-\Delta d,\qquad D'=D+\Delta d.
```

A positive $`\Delta d`$ is a draw; a negative value is a repayment. The sum
$`H+D`$ is unchanged. Progress, circulating supply, and the price schedule also
stay fixed. An advance does not create surplus for a floor raise.

The market must cover the resulting transfer from held reserve. Backing coverage
by $`H+D`$ does not waive that liquidity check.

## Combined trading and advance settlement

A holder can choose target AVA quantity $`n'`$ and target advance $`d'`$ in the
same action. Let $`C`$ be the signed gross trade transfer **to the market**:
positive for a purchase, negative for a redemption, and zero when holdings do
not change. The trade uses the [ordinary AVM quote](AVM-Specification).

Then

```math
H'=H+C-\Delta d,\qquad D'=D+\Delta d,
```

and the net reserve transfer to the holder is

```math
V_{\rm holder}=-C+\Delta d-f_{\rm trade}-f_{\rm draw}.
```

A negative $`V_{\rm holder}`$ means the holder pays reserve. The target must
satisfy $`d'\le Fn'`$ and $`H'\ge0`$. Settlement also checks the holder's
ability to pay and their minimum acceptable final reserve balance.

This permits a purchase to supply collateral for a simultaneous draw and a sale
to supply proceeds for repayment. Merely selling collateral does not silently
reduce principal: the submitted target and resulting settlement must satisfy the
collateral rule.

## Fees and rounding

The draw fee is charged only on $`\max(\Delta d,0)`$ at the configured
reserve-access rate. Repayment incurs no separate fee. A simultaneous purchase
or redemption has its own trade fee, calculated separately.

Charges use conservative upper bounds, payouts lower bounds, and advance
capacity is rounded down. Fees round upward under the
[AVM numerical rules](AVM-Specification#numerical-and-settlement-rules). A draw
fee is paid in addition to its backing movement; it is not added to principal.

## Resting positions and custody

Outstanding principal has no interest accumulation or maturity clock. With
unchanged $`n`$ and nondecreasing $`F`$, the capacity inequality remains
satisfied. The holder can leave the position encumbered or repay to reduce the
encumbrance.

The application must bind each position to its owner and market, keep actual AVA
collateral under the required controls, reconcile market-wide advance totals,
and settle the net payment, fees, and new state atomically. Those duties are
necessary for the accounting model to describe actual assets.

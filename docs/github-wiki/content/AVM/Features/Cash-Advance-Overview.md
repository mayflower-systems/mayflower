<!--
title: Cash Advance Overview
summary:
  Reserve advances against AVA floor value, their purpose, and their limits.
tags:
  - avm-features
  - cash-advance
-->

A cash advance releases reserve against the floor value of an AVA position.
The holder keeps the AVA position, but its collateral remains encumbered by the
outstanding advance. The facility uses the same reserve asset as the market;
it has no separate lender pool.

This page explains the economic rationale. [Cash Advance Mechanics](Cash-Advance-Mechanics)
specifies the accounting and settlement rules.

## Why the floor supports an advance

For $n$ AVA at floor price $F$, the gross floor value is $Fn$. An advance of
$d\le Fn$ releases part of this value now. The holder cannot also withdraw the
same backing by freely transferring or redeeming the collateral.

The reserve amount moves from liquid holdings to an outstanding collateralized
advance. Total backing is unchanged. This explains both the capacity limit and
the need to distinguish backing from available cash.

## No time-based interest

Outstanding principal does not accrue interest in this design. A newly drawn
amount incurs a configured reserve-access fee; repayment has no separate fee.
There is no time-based maturity or liquidation trigger in the accounting rules.

With fixed collateral quantity, a floor that stays fixed or rises cannot reduce
the position's advance capacity. This removes the usual price-fall trigger for
undercollateralization in the reserve denomination. It still depends on correct
collateral custody, authorized state transitions, and the reserve asset itself.

## What the holder receives and retains

Suppose an AVA position has a floor value of 90 reserve units. A draw of 90 pays
90 less the reserve-access fee, if liquid reserve is available. The holder
retains the position's remaining economic exposure, with 90 units of advance
principal to repay before the position can be released unencumbered.

A later floor increase can increase capacity or reduce the minimum collateral
quantity needed for the same principal. It does not automatically transfer more
reserve or repay the existing advance.

The facility can be useful when a holder wants reserve liquidity while retaining
a position. Full utilization is a choice, not a universal optimal strategy:
fees, collateral restrictions, liquidity needs, and the risks of deploying the
released capital all matter. Receiving advance proceeds is not by itself an
unencumbered investment profit.

## Repayment and exit

The holder can repay principal to restore held reserve and reduce the
encumbrance. A sale and repayment can be settled together, using sale proceeds
to help repay the advance. The resulting position must still satisfy its
collateral constraint, and the market must have enough liquid reserve for the
net payout.

A holder may leave a collateralized position outstanding without an interest
clock increasing the principal. That does not cancel the advance or create a
second claim on its collateral. Legal classification, tax treatment, and any
additional application lifecycle are outside these accounting rules.

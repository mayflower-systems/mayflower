<!--
title: Mayflower
summary:
  An open-source protocol for reserve-backed markets, built around the Amplifying
  Value Machine. An introduction to the AVM and the components developers can
  build on.
tags:
  - mayflower
-->

Mayflower is an open-source protocol for reserve-backed asset markets. At its
core is the **[Amplifying Value Machine](Amplifying-Value-Machine) (AVM)**: an
automated market maker that combines price discovery with a reserve-backed
redemption floor that can rise as the market evolves.

Anyone can build on Mayflower. Developers can use its components to create
markets, integrate them into applications, and design services around trading,
collateral, participation rewards, and revenue distribution.

## The Amplifying Value Machine

An AVM market issues an **Amplified Value Asset (AVA)** against a chosen reserve
asset. Purchases add capital to the reserve; redemptions return capital from it.
The pricing schedule determines the cost of issuance and the proceeds of
redemption. Buyers fund the market's liquidity as they participate, so operating
the market does not require a separate set of liquidity providers.

The AVM's defining feature is its **redemption floor**. Alongside a changing
market price, each unencumbered AVA has a minimum redemption value in the
reserve asset, before fees. The floor stays fixed or rises under the market's
rules. [Floor raising](AVM-Floor-Raising) reshapes the pricing schedule to
support a higher floor while preserving the live price and respecting the
market's backing constraints. This lets a market convert part of its
price-discovery value into a higher minimum claim on its reserve.

The floor is denominated in the reserve asset: a floor measured in SOL remains a
claim in SOL. A purchase above the floor still has exposure to the market's
price movements.

The same foundation supports additional features:

- **[Cash advances](Cash-Advance-Overview):** access reserve liquidity against
  the floor value of pledged AVA holdings.
- **[Call options](Call-Options):** acquire AVA through floor-priced exercise.
- **[Floor investment](Floor-Investment):** use surplus backing to support a
  higher redemption floor.

Start with the [AVM overview](Amplifying-Value-Machine) and
[design rationale](AVM-Rationale). The [specification](AVM-Specification)
explains pricing and backing, and [market evolution](AVM-Evolution) walks
through how trades and floor raises change a market. The
[Linear AVM](Linear-AVM) provides a concrete curve model.

## Build on Mayflower

Mayflower's open-source components are available under the Apache 2.0 license.
Developers can build trading interfaces, market analytics, simulations, and
applications that compose the AVM with other on-chain services.

[Protocol scope](Protocol-Scope) explains how the components fit together and
distinguishes permissionless public operations from owner-authorized actions and
deployment-specific configuration. Anyone can build on the algorithms; a
particular deployment's controls must be described separately.

Use the [AVM specification](AVM-Specification) to understand market state,
pricing, and reserve accounting, then follow [market evolution](AVM-Evolution)
for the effects of individual actions. [AVM features](AVM-Features) describes
the optional mechanisms an application can combine with its market.

This wiki separates the rules a system must enforce from the rationale for
choosing them. Specification pages define state, equations, and valid
transitions; rationale pages explain incentives, tradeoffs, and the assumptions
behind the mathematical properties.

## The wider system

Mayflower also includes components for rewarding participation and distributing
funded revenue. In the intended integration, AVM actions transfer their protocol
fees directly into reward-clearing pots in the market's base reserve asset.
Participants earn credits through asset-weighted referral participation and
spend them in those auctions. Fees can therefore be distributed in their
original assets without a central liquidation desk or protocol-supplied dollar
prices.

These components complement the AVM's market mechanics:

| Component                              | Purpose                                                                                                                                                               |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [Mayflower Credits](Mayflower-Credits) | Participation rewards that holders can use for emission weighting, referral incentives, and revenue clearing.                                                         |
| [Credit Emissions](Credit-Emissions)   | A scheduled credit budget, emitted at a continuously declining rate and allocated among eligible asset channels.                                                      |
| [Referral Network](Referral-Network)   | Allocates credits to participating asset holders and their uplines, with parent-funded incentives that give newcomers referral offers to compare.                     |
| [Revenue Clearing](Revenue-Clearing)   | Auctions funded reward pots to credit holders. Credits still committed at finalization are burned, and surviving weighted shares determine each participant's payout. |

Together, these components connect market activity, participation incentives,
and access to revenue. Each linked overview introduces its component and leads
to its specification and rationale.

<!--
title: Amplifying Value Machine
summary:
  Reserve-backed issuance and redemption with a floor that can rise under verified rules.
tags:
  - avm
  - pricing
-->

The Amplifying Value Machine (AVM) is an automated market maker for
reserve-backed assets. A buyer supplies the reserve asset and receives an
**Amplified Value Asset (AVA)**. A seller redeems AVA against that reserve.
A deterministic pricing schedule sets the exchange amounts, and purchases fund
the reserve used by subsequent redemptions.

An AVA has a minimum redemption value denominated in its market's reserve asset,
before fees and subject to any outstanding advance secured by the position.
This **floor** remains fixed or rises under the market's rules. A valid
[floor raise](AVM-Floor-Raising) increases it without changing the live market
price, while preserving the required backing constraints.

The floor is a reserve-denominated amount. It does not guarantee the reserve
asset's value in another currency or recover the premium paid by someone who
bought above the floor. Floor raises depend on available geometry or surplus
and an accepted proposal; their occurrence or rate is not guaranteed.

## The market in three parts

- **Pricing:** a floor, a transition ramp, and a main schedule determine issuance
  costs and redemption proceeds.
- **Backing:** the market accounts for the capital required by circulating AVA.
  Held reserve and collateralized advances must cover that requirement, and
  payouts must have sufficient liquid reserve.
- **Evolution:** trades move pricing progress; redemptions may contract the
  schedule, while floor raises can turn part of price-discovery value into a
  higher minimum reserve claim.

Buying supplies the market's liquidity, so the core mechanism does not depend
on separately recruited liquidity providers. This makes the reserve and pricing
rules central to the market's operation. [AVM Rationale](AVM-Rationale)
explains the motivation and the limits of these properties.

## Read the specification

| Page                                   | Contents                                                                    |
| -------------------------------------- | --------------------------------------------------------------------------- |
| [AVM Specification](AVM-Specification) | State, pricing, circulation, backing, and settlement rules                  |
| [AVM Evolution](AVM-Evolution)         | Changes caused by purchases, redemptions, contraction, and floor operations |
| [AVM Floor Raising](AVM-Floor-Raising) | Proposal geometry, backing conservation, and numerical acceptance           |
| [Linear AVM](Linear-AVM)               | A concrete characteristic with closed-form expressions and examples         |
| [AVM Features](AVM-Features)           | Cash advances, floor investment, and optional floor-priced issuance         |

## Relationship to the wider system

The AVM determines market pricing and backing. [Credit emissions](Credit-Emissions)
and the [referral network](Referral-Network) allocate participation rewards.
[Channel weighting](Emission-Channel-Weighting) directs credit budgets among
eligible markets; it does not alter an AVM's pricing schedule or grant market
parameter authority. [Revenue clearing](Revenue-Clearing) distributes funded
reward pots, separately from assets retained as market backing.

<!--
title: Credit Emissions
summary:
  How the emission schedule creates a budget, channel pledges route it to
  channels, and referral participation turns allocated rewards into newly minted
  credits.
tags:
  - tokenomics
  - emission
-->

The credit emission system distributes newly issued
[Mayflower credits](Mayflower-Credits) to participants over time. A single schedule
determines the amount available; holders' channel pledges divide it among
channels; eligible participation and the [Referral Network](Referral-Network)
determine who earns each channel's allocation. A member collects its final
retained reward, and that collection mints the corresponding credits.

A **channel**, also called an **outlet**, is a market's share of the common
emission stream. The referral network has one shared tree. Each channel
accounts separately for its members' points, rewards, and voluntary sharing;
channel weights select how much emission reaches each channel, not who belongs
to the network.

## The allocation sequence

1. **Schedule:** An annualized emission rate sets the stream of newly accrued
   entitlement. The rate follows a non-increasing, whole-second decay schedule.
2. **Channel weighting:** credit holders pledge credits to channels. When any
   channel has weight, channels receive emission in proportion to their pledged
   weights. When all weights are zero, registered channels share equally.
3. **Participation:** In each channel, eligible members' deposited collateral
   supplies points. The channel's allocation is attributed in proportion to
   those points; each point enters the denominator once.
4. **Referral distribution:** Each source's attribution is divided between the
   source and its upline. Members may then voluntarily share part of their
   earnings with their direct children. The same network relationships apply
   across channels, while the points and sharing allocations are
   channel-specific.
5. **Collection:** Each member's retained entitlement is collected and minted as
   credits. An amount attributed to a source before referral is an intermediate
   calculation, not a second claim.

Each stage divides the same scheduled budget. Channel pledges, referral overrides,
and voluntary sharing do not create extra emission. If there are no registered
channels, or a channel has no eligible points, the corresponding allocation is
recorded as unallocated for that interval; later participants do not inherit it.

## Accrual and supply

Scheduled emission is an accounting entitlement, not a quantity automatically
minted as time passes. After the mint authority has been transferred to Reserve,
collecting a final retained reward is the ongoing issuance path. A bootstrap
balance can already exist before that handoff; it is separate from scheduled
accrual. Channel pledges use already-issued credits and do not mint more.

Positive decay reduces the rate over time; zero decay holds it constant. A
change in the decay coefficient starts from the rate already in force, so the
rate never jumps upward. A fixed positive decay has a finite ideal remaining
tail, while a positive rate held at zero decay can continue indefinitely. The
[clearing auction](Revenue-Clearing) consumes credits separately; the balance
between issuance and clearing affects circulating supply.

## Chapters

- [Emission Schedule](Emission-Schedule) — The decaying rate,
  cumulative entitlement, and prospective changes to decay.
- [Channel Weighting](Emission-Channel-Weighting) — How pledges route
  the stream, including the zero-weight and empty-channel cases.
- [Participation and Referral Rewards](Participation-and-Referral-Rewards) —
  Eligible points, upline overrides, mutualization, and final retained rewards.
- [Accrual, Collection, and Conservation](Emission-Accounting) —
  Time-varying reward indexes, unallocated amounts, rounding, and how collection
  creates supply.

## How it connects

The [Referral Network](Referral-Network) explains the underlying attribution
mechanism independently of emissions.
[Long-Term Economics of Credits](Long-Term-Economics-of-Credits) describes the
incentives created by channel weighting, emission, and revenue clearing.

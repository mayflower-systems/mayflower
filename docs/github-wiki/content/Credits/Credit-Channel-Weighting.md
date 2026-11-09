<!--
title: Credit Channel Weighting
summary:
  Holders pledge credits to channels, directing the proportion of future
  emission each channel receives.
tags:
  - tokenomics
  - emission
-->

A holder of [Mayflower credits](Mayflower-Credits) can pledge credits to a channel to
direct a portion of future [emission](Credit-Emissions) to that channel. A
channel, or asset lane, has its own eligible asset balances and reward accounting
within the shared referral network.

Channel allocation is a separate layer from the
[referral specification](https://github.com/mayflower-systems/mayflower/blob/a1f87bfa8469630444626ddbc11536512c78bb16/referral/rust/mayf-referral/SPEC.md).
The kernel accounts for chronological gauge changes and implements credit-weighted
family marketing as a separate allocation rule. The surrounding programs enforce
custody and authorization for the credits pledged to each mechanism.

## How a pledge changes allocation

Each channel's weight is the total credits pledged to it. When the total across
all channels is positive, a channel receives the fraction of the scheduled global
stream represented by its weight divided by the total weight. For example, if
two channels have weights of 75 and 25 credits, they receive 75% and 25% of
subsequent emission in ideal arithmetic. A channel with no weight receives none
while another channel has weight.

A channel with no eligible assets receives its assigned allocation as unallocated
budget rather than passing it to the other channels. When no channel has weight,
the unavailable allocation is unallocated and unissued. Neither case creates
backpay for the first later participant or pledger.

Weights divide the emitter's global rate `E(t)`, which is planned to decrease
continuously. They do not choose its decay curve, create additional emission, or
compare raw quantities of different assets. Over each interval the lanes share
the budget accumulated from that time-varying rate.

## Holding and changing a pledge

The protocol holds already-issued credits for the selected channel until their
owner reduces the pledge. Increasing or reducing a pledge changes future
allocation after the preceding interval is settled. Current weights cannot be
applied retroactively to earlier emission.

The reference integration uses exclusive custody: pledged credits cannot also
back another channel pledge, a family marketing stake, a clearing commitment, or
an auction bid. Release them from their current commitment before assigning them
elsewhere. Pledging, reducing a pledge, and moving released principal do not mint
credits or change their supply. Dual-use voting power over family escrow would
require a separate integration rule.

Pledging credits does not require referral membership. A gauge pledge changes a
lane's allocation; it does not itself earn a family marketing share. That share
requires separate credit escrow under a member's immediate parent. The
[Referral Network](Referral-Network) divides each lane's allocation among asset
holders and their ancestors, with optional marketing funded only from referral
commissions. See
[Channel Weighting](Emission-Channel-Weighting) for the formulas and
timing rules.

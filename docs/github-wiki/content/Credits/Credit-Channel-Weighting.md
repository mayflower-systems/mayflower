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
channel corresponds to a market and has its own participation points and reward
accounting within the shared referral network.

## How a pledge changes allocation

Each channel's weight is the total credits pledged to it. When the total across
all channels is positive, a channel receives the fraction of the scheduled
stream represented by its weight divided by the total weight. For example, if
two channels have weights of 75 and 25 credits, they receive 75% and 25% of
subsequent emission in ideal arithmetic. A channel with no weight receives none
while another channel has weight.

A channel with no eligible points still receives its allocation, but that
amount is unallocated within the empty channel rather than redistributed to the
others.

## Holding and changing a pledge

The protocol holds pledged credits for the selected channel until their owner
reduces the pledge. The same credits cannot simultaneously back another pledge
or be committed to revenue clearing. Increasing or reducing a pledge changes
future allocation immediately after the prior interval is settled; it does not
change earlier allocation or the total emission schedule.

Pledging credits does not require referral membership. Channel weights determine
the amounts routed to channels; the [Referral Network](Referral-Network)
divides each channel's allocation among eligible members. See
[Channel Weighting](Emission-Channel-Weighting) for the formulas and
timing rules.

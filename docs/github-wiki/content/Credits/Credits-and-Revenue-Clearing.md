<!--
title: Credits and Revenue Clearing
summary:
  Participants commit credits to an epoch-based clearing auction and receive
  shares of its funded revenue pot.
tags:
  - tokenomics
  - revenue-clearing
-->

[Mayflower credits](Mayflower-Credits) are the clearing asset used in Mayflower's
[revenue-clearing auction](Revenue-Clearing). Participants commit credits
during an epoch to claim shares of a pot funded with a reward asset. The reward
asset can differ from credits.

The [referral specification](https://github.com/mayflower-systems/mayflower/blob/a1f87bfa8469630444626ddbc11536512c78bb16/referral/rust/mayf-referral/SPEC.md)
defines distinct family-escrow and admission uses for issued credits, implemented
by the referral kernel. Those uses do not alter the clearing rules below.

## Committing and spending credits

A deposit earns shares according to when it is made; an earlier deposit can
receive more shares per credit. Before the deadline, a participant can withdraw
from its newest surviving deposit lot. It gives up the corresponding shares,
receives the unpenalized portion, and irreversibly forfeits the withdrawal
penalty.

At the deadline, deposits, withdrawals, and funding stop. Finalization
irreversibly forfeits all remaining committed credits and fixes the reward pot
and surviving shares. Each position can then claim its proportional share of the
pot. Forfeited credits enter burn-only custody; a separate burn reduces reported
mint supply.

A participant that withdraws all of its deposits has no surviving shares, though
its withdrawals may have incurred penalties. The exact entry and exit formulas
are described in [Time Weight](Revenue-Clearing-Time-Weight).

## How clearing relates to emission

Clearing consumes already-issued credits. It does not set the emitter's planned
continuously decreasing rate `E(t)`. The [referral allocation model](Credit-Emissions)
divides the scheduled budget, and newly accrued emission does not itself fund a
clearing pot. Uncollected referral rewards must first be collected into issued
credits before those credits can be committed to clearing.

The reference integration uses exclusive custody across channel pledges, family
escrow, clearing commitments, and auction bids. A credit in family escrow must
first be withdrawn with its prior family earnings settled; a channel pledge
must first be reduced. An auction bid must be released under that auction's
rules before its principal can be used elsewhere.

Family escrow is different from clearing: its principal remains withdrawable
without a mandatory penalty or forfeiture. Ordinary referral admission transfers
a credit fee to the parent, while a scarce top-level seat's winning auction
payment burns credits. Neither payment creates clearing shares or funds the
clearing pot.

Collections add issued supply. Transfers and refundable custody movements do not
change it. Clearing forfeitures remove credits from spendable circulation; their
subsequent burn reduces mint supply. Root-auction payment burns also reduce
supply. Neither burn authorizes replacement emission.

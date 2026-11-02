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

Clearing consumes already-issued credits. It does not change the
[emission schedule](Emission-Schedule), and newly accrued emission
does not itself fund a clearing pot. A credit pledged to a channel must first be
released from that pledge before it can be committed to clearing. Issuance
through collected rewards and forfeiture through clearing are separate flows
affecting the supply of credits.

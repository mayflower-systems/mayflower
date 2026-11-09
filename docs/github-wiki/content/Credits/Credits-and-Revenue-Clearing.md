<!--
title: Credits and Revenue Clearing
summary:
  Participants commit credits to an epoch-based clearing auction and receive
  shares of its funded revenue pot.
tags:
  - tokenomics
  - revenue-clearing
-->

[Mayflower credits](Mayflower-Credits) are the clearing asset used in
Mayflower's [revenue-clearing auction](Revenue-Clearing). Participants commit
credits during an epoch to claim shares of a pot funded with a reward asset. The
reward asset can differ from credits.

In the intended AVM integration, market actions synchronously fund these pots
with their protocol-fee allocations in the base reserve asset. Each pot keeps
that asset denomination. Credits provide a common bidding unit across the pots;
the clearing formula does not need dollar prices or asset conversion. The
[rationale](Revenue-Clearing-Rationale) explains this choice.

The [referral specification](Referral-Network-Formal-Specification) defines
separate family-escrow and admission uses for issued credits. The
[clearing specification](Revenue-Clearing-Formal-Specification) defines the
auction; using credits for one purpose does not give them weight in another.

## Committing and spending credits

A deposit earns shares according to when it is made; an earlier deposit can
receive more shares per credit. Before the deadline, a participant can withdraw
from its newest surviving deposit lots, consuming them in newest-first order. It
gives up the corresponding shares and receives the refund computed once for the
total withdrawn amount. The remaining withdrawn principal is the penalty and is
burned during that withdrawal.

At the deadline, deposits, withdrawals, and funding stop. Finalization burns all
remaining committed credits and fixes the reward pot and surviving shares. Each
position can then claim its proportional share of the pot, rounded down to whole
reward base units. The burn happens even if the funded pot is zero.

Claims use the same frozen totals, so claim order does not change entitlement.
Unclaimed rewards and rounding residuals remain in the epoch vault. If no shares
survive, no position is eligible to claim through this algorithm. The current
lifecycle has no recovery or remainder-sweep operation; it does not
automatically transfer those funds to protocol custody.

A participant that withdraws all of its deposits has no surviving shares, though
its withdrawals may have incurred penalties. The exact entry and exit formulas
are described in [Time Weight](Revenue-Clearing-Time-Weight).

## How clearing relates to emission

Clearing consumes already-issued credits. It does not set the emitter's planned
continuously decreasing rate $E(t)$. The
[referral allocation model](Credit-Emissions) divides the scheduled budget, and
newly accrued emission does not itself fund a clearing pot. Uncollected referral
rewards must first be collected into issued credits before those credits can be
committed to clearing.

The reference integration uses exclusive custody across channel pledges, family
escrow, clearing commitments, and auction bids. A credit in family escrow must
first be withdrawn with its prior family earnings settled; a channel pledge must
first be reduced. An auction bid must be released under that auction's rules
before its principal can be used elsewhere.

Family escrow is different from clearing: its principal remains withdrawable
without a mandatory penalty or forfeiture. Ordinary referral admission transfers
a credit fee to the parent, while a scarce top-level seat's winning auction
payment burns credits. Neither payment creates clearing shares or funds the
clearing pot.

Collections add issued supply. Transfers and refundable custody movements do not
change it. Clearing burns reduce supply during withdrawal or finalization;
root-auction payment burns also reduce supply. Neither burn authorizes
replacement emission. Burning credits does not itself create the reward asset or
fund a revenue pot.

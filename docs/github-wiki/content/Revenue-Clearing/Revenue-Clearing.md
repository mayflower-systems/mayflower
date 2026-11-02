<!--
title: Revenue Clearing
summary:
  A public, epoch-based auction that distributes a reward pot among participants
  who commit clearing tokens, using time-weighted entry and a time-dependent
  exit penalty.
tags:
  - revenue-clearing
  - tokenomics
-->

Revenue clearing is an auction for distributing a pot of rewards in exchange for
a **clearing token**. During a fixed period called an **epoch**, participants
commit clearing tokens and receive weighted shares of the pot. Earlier
commitments can earn more shares per token. Before the deadline, a participant
can withdraw, surrendering the corresponding shares and forfeiting a portion of
the withdrawn tokens. At finalization, all remaining committed tokens are
irreversibly forfeited, and the net reward pot is divided among the surviving
shares.

In Mayflower, [Mayflower credits](Mayflower-Credits) are the clearing asset and protocol
revenue can fund the reward pot. The auction itself is a general mechanism: the
reward asset, the source of funding, and the clearing token can be chosen for a
particular use.

## Why a public auction needs structure

A public blockchain does not by itself provide a trusted auctioneer who can
collect sealed bids, reveal them simultaneously, and decide who arrived first in
an economically neutral way. Transactions and bids can be observed before or
during execution, and their ordering can affect the outcome. Participants can
use multiple addresses, gain information at different times, or change their
position while an auction is live. A design that depends on hidden bids, equal
information, or one identity per person cannot take those assumptions for
granted.

Revenue clearing makes participation and its consequences explicit. Everyone
uses the same epoch deadline, entry-weight formula, withdrawal-penalty formula,
and settlement rule. A participant can enter or leave at any point in the open
window, with the terms determined by the time of that action. The auction does
not depend on a trusted party choosing a clearing price or privately deciding an
allocation.

This is **fairness of rules and accounting**: the same action at the same time
and state receives the same treatment, and each surviving share receives the
same pro-rata entitlement before whole-unit rounding. Participants still choose
different times and amounts and may possess different information. Public rules
do not make transaction ordering or every participant's opportunity identical;
they make the effect of each accepted action computable.

## The pieces

- **Clearing token:** the asset participants commit. A participant's committed
  amount is called its principal.
- **Reward asset and pot:** the asset supplied for distribution. It can be
  different from the clearing token. Funding adds to the pot during the epoch,
  after any funding fee.
- **Epoch:** a window of duration $T$. Deposits, withdrawals, and funding are
  accepted before its deadline; settlement follows the deadline.
- **Share:** a unit of entitlement to the pot. A deposit earns shares according
  to its entry time. Shares are accounting units: their value depends on the
  final pot and the total surviving shares.
- **Deposit lot:** a record of one deposit and its shares. Each deposit keeps
  its own entry weight. A position is a participant's collection of lots for one
  auction epoch.
- **Funder and participant:** a funder supplies rewards; a participant commits
  clearing tokens. The same actor can play both roles.

## The auction in five steps

1. Funders add rewards to the epoch pot. A funding fee, if configured, is
   separated from the amount credited to the pot.
2. Participants deposit clearing tokens. A deposit at elapsed time $t$ earns
   shares using the epoch's entry weight $w(t)$.
3. Before the deadline, participants can withdraw from their newest surviving
   lot. The withdrawal removes shares and returns the principal remaining after
   the penalty $p(t)$; the penalty is irreversibly forfeited.
4. At the deadline, deposits, withdrawals, and funding stop. Finalization
   irreversibly forfeits remaining principal and fixes the net reward pot and
   surviving shares.
5. Each position claims its proportion of the fixed pot. Payouts round down to
   whole reward units. After all shares are claimed, the remaining rounding
   units are transferred to protocol custody. If no shares survive, the pot goes
   there at finalization.

Deposits increase the number of shares competing for a fixed pot. Withdrawals
destroy shares, increasing the remaining positions' fractions of that pot. A
forfeited withdrawal penalty is clearing principal; it does not add to the
reward pot.

## The basic equations

Let $t$ be elapsed time in an epoch of duration $T$. The epoch fixes an **entry
bonus slope** $b$ and an **exit penalty exponent** $k$:

$$
w(t)=1+b\left(1-\frac{t}{T}\right),\qquad 0\le b\le2,
$$

$$
p(t)=\left(\frac{t}{T}\right)^k,\qquad k\in\{1,2,3,4\}.
$$

In ideal arithmetic, a deposit of $a$ clearing tokens earns $a\,w(t)$ shares. A
withdrawal of $x$ tokens returns $x(1-p(t))$ and forfeits $xp(t)$, while
removing the corresponding shares from its lot. The entry weight is fixed at
deposit; the exit penalty is calculated from time since the epoch began. Actual
accounting uses directed rounding described in the
[formal specification](Revenue-Clearing-Formal-Specification).

If the finalized net pot is $R$, the surviving total is $S$, and a position has
$S_i$ shares, its payout is

$$P_i=\left\lfloor\frac{R S_i}{S}\right\rfloor_{\text{reward unit}}.$$

$R/S$ is reward per weighted share. A participant's reward per clearing token
also depends on entry weight, withdrawals, and rounding. The pot and surviving
shares arise from the actions taken during the epoch; there is no preset
clearing-token price.

## Notes

- [Rationale](Revenue-Clearing-Rationale) — Why this mechanism is useful on a
  public blockchain and how its rules shape participation and fairness.
- [Time Weight](Revenue-Clearing-Time-Weight) — Entry, exit, deposit lots, and
  a worked example from deposit through settlement.
- [Formal Specification](Revenue-Clearing-Formal-Specification) — Parameters,
  accounting transitions, rounding, and conservation.

## How it connects

- [Emission](Emission-Channel-Weighting) allocates credits to
  participants through channel weights and the
  [referral network](Referral-Network). Revenue clearing uses committed credits
  to claim a portion of a funded reward pot.
- Clearing and emission are separate processes: clearing forfeitures do not
  change the [emission schedule](Emission-Schedule). Their combined
  effects contribute to the
  [long-term economics of credits](Long-Term-Economics-of-Credits).

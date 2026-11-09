<!--
title: Referral Network
summary:
  A finite-depth referral tree that allocates asset-funded credits to members
  and their uplines, with discretionary credit-weighted family marketing.
tags:
  - referral
  - tokenomics
-->

The Referral Network divides a scheduled stream of [Mayflower credits](Mayflower-Credits)
among eligible asset holders and their referral ancestors. A parent can then
spend some of its referral commissions on a marketing offer for its immediate
children. Those children divide the offer by escrowing credits.

The global emission rate decreases continuously; the cumulative amount available
to distribute continues growing while emission is positive. The tree divides
each interval's scheduled budget without increasing it.

This wiki describes the design in the
[implementation specification](https://github.com/mayflower-systems/mayflower/blob/a1f87bfa8469630444626ddbc11536512c78bb16/referral/rust/mayf-referral/SPEC.md).
The kernel implements the allocation rules through pure actions. Deployment
provides authenticated state, the emission schedule, auctions, and token custody.

## One tree, two forms of admission

The logical protocol root earns nothing. Beneath it are a small number of scarce,
auctioned earning seats at depth 0. A winning seat payment is made in credits and
**burned**. The auction and the credit inventory needed to bootstrap the first
paid auction are separate protocol responsibilities.

An ordinary member instead selects an existing parent and pays that parent's
current join fee in credits. This payment **belongs to the parent**. A parent can
set a zero fee, and can change its offer for future joiners.

The chosen parent connection is permanent. Deposits and escrow balances can
change, but an existing connection cannot move, be severed, or have another node
inserted into it. A node's *upline* is its chain of earning ancestors; its
*downline* is its entire descendant subtree.

## Asset participation funds the core allocation

Each asset has its own emission lane. An external allocator, such as gauge voting,
divides the global credit budget among lanes. Each lane compares eligible asset
balances within that lane; it does not add raw BTC, ETH, and SOL quantities.

At depth $d$, a member retains $1-dq$ of the gross attribution from its own
eligible assets. Each of its $d$ earning ancestors receives a fixed fraction $q$.
The maximum depth $H$ and override rate $q$ are fixed, with

$$q>0,\qquad Hq<1.$$

For example, at depth 2 with $q=10\%$, a source's gross attribution is divided
80% to its holder, 10% to its parent, and 10% to its grandparent. The deposited
principal is not divided. A grandparent benefits from descendant assets even if
the intervening parent has no assets of its own.

A node at depth $H$ cannot recruit children, but retains a positive own-asset
share and can participate in its parent's family marketing pool.

## Credits divide the optional family budget

A parent chooses a marketing fraction $\mu$ between 0% and 100%. It shares that
fraction of its **gross downline override income**, aggregated in credits across
all asset lanes. Its own-asset earnings and incoming gifts are outside this
budget.

Immediate children divide the budget in proportion to their escrowed credits.
Their assets, number of accounts, and descendants' escrow are not the weights.
A received gift stays with its recipient; it is not shared onward or charged
another upline commission. If no child has credits escrowed, the parent keeps
the otherwise undistributed budget.

This gives families distinct offers. A productive downline can fund a larger
pool, while more competing child escrow reduces each staker's fraction. A parent
can change $\mu$ at any time for future accrual. It is a discretionary marketing
budget, not a permanent yield commitment. [Mutualization](Mutualization) explains
the mechanism.

## Balances change; history does not

Buying, selling, depositing, and withdrawing assets affect subsequent core
allocation. Staking or withdrawing credits affects subsequent family gifts.
Neither action changes the parent connection or earns rewards from an earlier
interval. Escrow principal is withdrawable without a mandatory lock period.

Before changing a family's marketing percentage or credit weights, accounting
must settle its earlier override income from every contributing lane. The
[emission accounting](Emission-Accounting) page defines that history and the
implementation requirements.

## Explore the model

| Page | What it explains |
| --- | --- |
| [Formal Specification](Referral-Network-Formal-Specification) | The allocation equations, conservation, and implementation boundary |
| [Mutualization](Mutualization) | Credit escrow, family gifts, and examples |
| [Link Shopping](Link-Shopping) | Comparing permanent positions and changing commercial offers |
| [Parameters](Referral-Network-Parameters) | Fixed network terms and participant choices |
| [Rationale](Referral-Network-Rationale) | Recruitment incentives, identity splitting, and economic limits |
| [Diagrams](Referral-Network-Diagrams) | Admission, commissions, and marketing flows |

The rules allocate the existing emission budget; they do not guarantee that
recruitment increases every member's absolute earnings. [Credit Emissions](Credit-Emissions)
explains the budget, and [Credit Channel Weighting](Credit-Channel-Weighting)
explains its allocation among assets.

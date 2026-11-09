<!--
title: Earning Credits
summary:
  Eligible assets fund core rewards and ancestor commissions; credit escrow
  weights family allocations from a parent's discretionary commission budget.
tags:
  - tokenomics
  - emission
-->

Participants earn [Mayflower credits](Mayflower-Credits) through the
[emission system](Credit-Emissions) and the
[Referral Network](Referral-Network). This page introduces the reward paths. The
[referral specification](Referral-Network-Formal-Specification) defines their
allocation equations, including credit-weighted family mutualization.

The emitter supplies a global rate $E(t)$ that is planned to decrease
continuously. The scheduled budget over an interval is the integral of that
rate. An external allocator, such as channel pledges, divides the budget among
asset lanes; it does not determine the emitter's decay curve. Each lane compares
only its own eligible asset balances. An unavailable allocation or an empty
lane's allocation is recorded as unallocated; later arrivals cannot claim it.

## Participation and referral rewards

Within a lane, each member's own eligible assets determine its gross
proportional attribution. A holder at earning depth $d$ keeps $1-dq$ of that
attribution; each of its $d$ earning ancestors receives the fixed commission
$q$. Auctioned top-level seats have depth zero. The protocol root earns nothing.

Maximum earning depth is $H$, with $Hq<1$. Bottom-level members cannot recruit
but retain a positive own-asset share. Commissions include assets held by
children, grandchildren, and every permitted further generation. A node can
therefore earn from its downline without holding assets itself.

Membership and eligible asset changes take effect prospectively. Neither joining
nor depositing creates rewards for earlier intervals. AVM use and channel
pledges do not require referral membership.

## Family marketing rewards

A parent may share a fraction $\mu$ of its **gross referral commissions** across
all lanes. Immediate children divide that pool in proportion to their issued
credits escrowed with the parent. Their own asset balances and descendant escrow
do not supply this weight. A child with no eligible assets can receive family
allocations.

The pool excludes own-asset income, admission fees, escrow principal, and
incoming family allocations. When no child has eligible escrow, the parent
retains its commissions. Family allocations are kept whole, without further
sharing or upline charges. These transfers divide existing emission rather than
creating another reward budget.

The parent may change $\mu$ for future accrual. Credit stakes can be withdrawn
after earlier income is settled, without a mandatory lock period or forfeiture.
Before changing $\mu$ or a child's escrow, all contributing lanes must reach a
common settlement cutoff under the old terms. A late staker receives no part of
earlier income. Family allocations do not automatically restake.

## From entitlement to supply

Elapsed emission records an entitlement rather than immediately issuing tokens.
A member collects its net reward, and collection issues the corresponding whole
credit base units once. Fractional earned entitlement remains accounted for.
Uncollected entitlement cannot simultaneously serve as issued credit escrow.

A creditless newcomer may join an available free link, earn credits from
eligible assets, and collect them before staking. A paid ordinary link requires
issued credits for the fee; the fee goes to the parent and is separate from
escrow. Before any earning seats exist, a credit-paid root auction requires
documented bootstrap supply. The system does not issue credits merely to fund
that payment.

See [Participation and Referral Rewards](Participation-and-Referral-Rewards) for
the allocation and examples, and
[Accrual, Collection, and Conservation](Emission-Accounting) for how reward
history and issuance are accounted.

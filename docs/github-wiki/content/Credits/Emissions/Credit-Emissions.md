<!--
title: Credit Emissions
summary:
  A declining global emission stream is routed to asset lanes, divided by
  referral rules, and optionally shared according to escrowed credits.
tags:
  - tokenomics
  - emission
-->

The emission system allocates one global stream of
[Mayflower credits](Mayflower-Credits), with a rate that declines continuously
over time. Asset participation generates core rewards and referral commissions.
Parents can share some of those commissions with their immediate children,
weighted by the credits those children escrow. Collection turns a member's net
earned entitlement into issued credits.

The [referral specification](Referral-Network-Formal-Specification) defines who
receives each lane's allocation. [Emission Accounting](Emission-Accounting)
defines how changing balances and terms are settled. The emission schedule,
asset eligibility, auction authority, and custody must provide authenticated
inputs and apply their resulting transfers atomically.

## One budget, separate decisions

1. **Global emission:** the continuously declining rate $E(t)$ establishes the
   credit budget by integration over time. The exact curve and its parameters
   belong to the emitter design.
2. **Lane allocation:** an external allocator, such as gauge voting, assigns
   that budget to asset lanes. In ordinary active operation, lane rates sum to
   $E(t)$. One active lane with no gauge competition receives the whole rate.
3. **Core referral allocation:** each lane divides its allocation using its own
   eligible asset balances. Every source funds its own retained share and a
   fixed commission for each earning ancestor.
4. **Mutualization:** a parent may share a fraction of its gross referral
   commissions across all lanes. Its immediate children divide that pool by
   their escrowed credits. Incoming family allocations are kept whole.
5. **Collection:** each member collects its net entitlement. The same amount is
   removed from its uncollected balance and minted once.

A lane, also called a channel, has its own asset identity, balance denominator,
and allocation history. Raw BTC and ETH quantities are never added together to
make a common asset weight. The [Referral Network](Referral-Network) is shared
across lanes; the common marketing weight is already-issued credit escrow.

Each stage divides the same scheduled budget. Gauge weights, commissions,
marketing, fees, and escrow do not create another emission stream. Unavailable
routing, allocations to empty lanes, and numerical residuals are accounted for
explicitly as unallocated, unissued budget. Later participants do not inherit
it.

## Membership and rewards

Auctioned top-level seats have earning depth zero. The logical protocol root
earns nothing. Ordinary members choose a permanent parent, and the maximum
earning depth is $H$. A node at that limit can earn rewards but cannot recruit
children. In the intended referral configuration, the fixed commission rate
$q>0$ satisfies $Hq<1$, leaving a positive own-asset fraction at every permitted
depth. The general rule also permits the zero configurations described in
[Parameters](Referral-Network-Parameters).

A member can earn commissions from actual descendant assets without holding
assets itself. It can also receive family allocations without holding assets,
provided it escrows issued credits with its immediate parent. Neither an empty
account nor an uncollected reward claim supplies marketing weight.

Parents set their current join fee and marketing percentage independently. A
marketing change applies prospectively at one common effective time across all
of the parent's contributing lanes. Assets and credit escrow can change
frequently without changing the parent connection.
[Emission Accounting](Emission-Accounting) explains the checkpoints needed to
preserve earlier rewards.

## Earned, issued, and escrowed credits

Scheduled emission and earned entitlement are not automatically minted as time
passes. Collection issues net earned credits. A documented bootstrap supply is
separate from this ongoing issuance path.

Family escrow holds already-issued credits as the child's withdrawable
principal. An ordinary join fee transfers issued credits to the parent; a
winning top-seat payment burns issued credits. Neither payment is family escrow.
Burning credits does not replenish the emission budget. Custody rules also keep
the same principal from simultaneously funding incompatible family, gauge,
clearing, or auction commitments.

## Chapters

- [Emission Schedule](Emission-Schedule) — The declining rate, cumulative
  budget, and the emitter-design boundary.
- [Emission Channel Weighting](Emission-Channel-Weighting) — The independent
  allocator and its chronological, budget-conserving interface.
- [Participation and Referral Rewards](Participation-and-Referral-Rewards) —
  Eligible assets, fixed upline commissions, credit-weighted marketing, and a
  worked allocation.
- [Emission Accounting](Emission-Accounting) — Lane and family indices,
  settlement across lanes, custody, collection, and numerical requirements.

See [Credit Channel Weighting](Credit-Channel-Weighting) for the allocator's
broader role and
[Long-Term Economics of Credits](Long-Term-Economics-of-Credits) for the
relationships between credit demand, emissions, and clearing.

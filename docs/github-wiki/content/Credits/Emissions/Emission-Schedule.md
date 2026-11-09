<!--
title: Emission Schedule
summary:
  A continuously declining global rate defines an increasing cumulative credit
  budget. Allocation and collection preserve each interval's scheduled amount.
tags:
  - tokenomics
  - emission
  - math
-->

The planned [credit emission](Credit-Emissions) rate **declines continuously over
time**. The schedule establishes how many credits are available over an interval;
channel allocation and referral rules decide who earns them. The target
accounting is specified in the
[implementation handoff](https://github.com/mayflower-systems/mayflower/blob/a1f87bfa8469630444626ddbc11536512c78bb16/referral/rust/mayf-referral/SPEC.md).

## Rate and cumulative budget

Let $E(t)\ge0$ be the continuously declining emission rate in credit base units
per unit of time, and let $t_0$ be the schedule's start. The ideal cumulative
scheduled budget is

$$B(t)=\int_{t_0}^{t}E(u)\,du,\qquad t\ge t_0.$$

For any $t_b\ge t_a\ge t_0$, the interval budget is

$$\Delta B=B(t_b)-B(t_a)=\int_{t_a}^{t_b}E(u)\,du.$$

The rate decreases while the cumulative budget continues to increase wherever
$E(t)>0$. A smaller flow still adds to the cumulative scheduled budget; it does
not subtract from credits scheduled earlier.

For example, if two successive intervals have integrated budgets of 300 and 250
credits, their combined budget is 550. Splitting them into more settlement calls
does not increase that entitlement. Implementations must take differences of
cumulative emission, rather than multiply the latest rate by a past interval.
A monotonic trusted clock and explicit finite-precision accounting are required.

The rate is global. It is not multiplied by the number of lanes, referral nodes,
or credit stakers. A family does not acquire an additional emission stream when
its parent enables marketing.

## Lane allocation follows the scheduled stream

An external allocator supplies each lane's rate $e_l(t)$. In ordinary active
operation:

$$\sum_l e_l(t)=E(t).$$

With one active lane and no gauge competition, that lane receives $E(t)$. Changing
gauge weights can move the scheduled stream between lanes, but cannot silently
reduce the global budget or replace the schedule with another rate policy.

If an allocation is unavailable, or a destination lane has no eligible assets,
the exception is recorded as unallocated and unissued. At the routing stage,
lane allocations plus unallocated routing amounts reconcile to the scheduled
budget. Within each lane, earned allocations and that lane's unallocated amounts
reconcile to the amount it received. An empty lane's past allocation is not saved
for its first later depositor. See [Emission Channel Weighting](Emission-Channel-Weighting).

## Scheduling is not minting

Elapsed time records a budget. Core allocation and marketing establish net
member entitlements. Successful collection removes an earned claim and issues
the same amount once. Fractional earned claims remain accounted for between
collections.

Already-issued credits can be transferred as admission fees, held in family
escrow, or burned in an authorized top-seat auction. These operations do not
alter scheduled emission. In particular, a burn does not reset the clock or
authorize replacement rewards. [Emission Accounting](Emission-Accounting)
separates the emission budget from token supply and custody.

## Emitter design and deferred supply analysis

The intended policy is continuous decline. Its exact curve, parameters, and
implementation belong to the emitter design; this page does not choose a decay
formula or add administrative controls. A constant rate can simplify an
allocation example, but it is not the planned global policy.

Continuous variation does not require settlement transactions at every instant.
The emitter supplies cumulative emission over intervals, and the accounting
checkpoints that history when relevant state changes.

A decreasing positive rate does not by itself prove that its integral over all
future time is finite. The far-future check that total token supply in base units
remains within `u64::MAX` is deferred. That analysis should use the eventual
emitter and supply rules; it is not resolved by asserting that emission declines.
Ordinary checked arithmetic for operational balances, indices, accrued claims,
and custody remains required.

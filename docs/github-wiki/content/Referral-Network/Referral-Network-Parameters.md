<!--
title: Referral Network Parameters
summary:
  Fixed depth and commission terms, external emission and auction boundaries,
  and the admission and marketing choices made by each node owner.
tags:
  - referral
  - parameters
-->

The [Referral Network](Referral-Network) separates fixed network terms from each
parent's configurable parameters. The following symbols and responsibilities
apply throughout these pages.

| Term                   | Meaning                                                                      | Who controls it                          |
| ---------------------- | ---------------------------------------------------------------------------- | ---------------------------------------- |
| $`H`$                  | Maximum earning depth                                                        | Fixed network configuration              |
| $`q`$                  | Commission to each earning ancestor                                          | Fixed network configuration              |
| $`E(t)`$               | Global credit emission rate, decreasing continuously                         | Global emission module                   |
| $`e_\ell`$             | Emission allocated to asset lane $`\ell`$                                    | External allocator, such as gauge voting |
| Top-seat supply        | Scarce depth-zero admission capacity                                         | Protocol auction authorization           |
| $`\mu_i`$              | Fraction of gross override income marketed to immediate children             | Node owner, prospectively                |
| Join fee and increment | Credit price for ordinary admission and its increase after a successful join | Parent, for future joins                 |
| $`p_{i,\ell}`$         | Own eligible asset balance in a lane                                         | Member's asset participation             |
| $`s_i`$                | Credits escrowed with the immediate parent                                   | Member's staking and withdrawal choices  |

## Depth and fixed commissions

The logical protocol root is non-earning. Auctioned seats have depth 0; their
children have depth 1. The deepest permitted member has depth $`H`$ and cannot
accept children.

The general allocation rule requires

```math
H\in\mathbb{Z}_{\ge0},\qquad 0\le q\le1,\qquad Hq\lt 1.
```

The intended referral configuration uses $`H\gt 0`$ and $`q\gt 0`$. Zero depth permits
only auctioned seats; zero commission disables ancestor rewards. Those valid
degenerate configurations do not provide the same recruitment incentives. A path
contains at most $`H+1`$ earning members, including its depth-zero seat.

A source at depth $`d`$ retains $`1-dq`$ of its gross own-asset attribution and
funds $`q`$ for each of its $`d`$ earning ancestors. The strict product bound
leaves positive retention at every valid depth.

For illustration, $`H=2`$ and $`q=10\%`$ give:

| Depth | Total sent to earning ancestors | Own-asset retention | Can recruit? |
| ----- | ------------------------------: | ------------------: | ------------ |
| 0     |                              0% |                100% | Yes          |
| 1     |                             10% |                 90% | Yes          |
| 2     |                20%, or 10% each |                 80% | No           |

These are example values, not deployment defaults. Increasing $`q`$ directs more
of each source's attribution to its ancestors. Increasing $`H`$ permits longer
chains but must preserve $`Hq\lt 1`$. Neither value changes through an ordinary
parent action once the network is operating.

The current accounting representation supports $`0\le H\le255`$ and up to 16
asset lanes. These are representation bounds, not required deployment values.
The configured depth must also fit the execution limits of the integrating
blockchain program.

## Marketing and admission choices

Each node starts with $`\mu_i=0`$ and can set it anywhere from 0 to 1. It
applies to gross override earnings from every lane, paid through one family pool
with a common credit-escrow denominator. Own-asset income, fees, and incoming
family allocations are excluded. A parent with no eligible child escrow keeps
its overrides.

A parent can change $`\mu_i`$ at any time for future accrual. All contributing
lanes must be checkpointed under the old terms before that change takes effect.
There is no separate effective date chosen by each lane's next settlement.

Ordinary join fees and increments are nonnegative credit amounts. Zero for both
is a valid default. A parent can edit them for future admissions, without
changing existing connections. The increment is a pricing convenience; it does
not prove that new accounts represent independent people.

[Link Shopping](Link-Shopping) explains how these choices combine into an offer.

## Emission and auction boundaries

The global emission module supplies a continuously decreasing rate $`E(t)`$. The
external allocator divides that scheduled stream among asset lanes; gauge
weights may change the relative shares without changing the global schedule. The
exact decay curve belongs to the emission module and is separate from the tree's
allocation rules. Empty or unavailable allocations must be recorded explicitly
rather than awarded retroactively to later depositors.

Top-level supply is scarce and controlled by the protocol auction. A winning
credit payment is burned; an ordinary join fee is paid to the parent. The
auction format, initial credit supply, numerical parameter values, and supported
lanes must be specified before deployment. They are not extra pricing
restrictions on ordinary parents.

The [formal specification](Referral-Network-Formal-Specification) gives the
allocation equations. [Emission Accounting](Emission-Accounting) covers changing
balances, shared family checkpoints, and finite-precision validation.

<!--
title: Participation and Referral Rewards
summary:
  Eligible asset balances weight scheduled credits and fixed upline commissions. Parents can share gross
  commissions across lanes with children weighted by their escrowed credits.
tags:
  - tokenomics
  - emission
  - referral
-->

After [channel allocation](Emission-Channel-Weighting), each lane divides its
assigned credits using eligible asset balances and the shared
[Referral Network](Referral-Network). Parents can then share part of their gross
referral commissions with children who escrow credits. The
[formal specification](Referral-Network-Formal-Specification) defines the rules;
this page gives their interval form and worked examples.

## Eligible balances and permanent membership

The host supplies an authoritative eligible balance $`p_{i,l}`$ for each member
$`i`$ in asset lane $`l`$. Each own balance enters that lane's total
$`P_l=\sum_i p_{i,l}`$ once. The asset-eligibility adapter must prevent
duplicate claims against the same underlying eligible position. Balances outside
the referral network do not receive core referral emission.

Only like asset units are added within a lane. A descendant aggregate helps
calculate commissions; it does not add the same assets to $`P_l`$ again.
Escrowed credits and uncollected earnings are not additional eligible asset
balances.

Auctioned top-level seats have earning depth zero. The logical protocol root
earns nothing and is not a paid ancestor. Every ordinary node chooses a
permanent parent. The maximum earning depth is $`H`$; members at that depth can
earn but cannot accept children. Asset balances can change without changing any
parent connection.

## Fixed upline commissions

The network fixes $`q`$ with $`Hq\lt 1`$; the intended referral configuration uses
$`q\gt 0`$. The interval formulas below assume unchanged eligible balances,
marketing fractions, and credit escrows. A change closes the old interval before
the new state takes effect; an unchanged total alone is not enough if members
trade offsetting positions.

For a lane allocation $`Y_l`$ over such an interval with $`P_l\gt 0`$, a source
holding $`p`$ has gross attribution

```math
G=Y_l\frac{p}{P_l}.
```

Here $`Y_l=\int_{t_a}^{t_b}e_l(t)\,dt`$ is the lane's integrated interval
budget. The global rate $`E(t)`$ may decline continuously during that interval;
the formula does not assume constant emission.

A source at depth $`d`$ retains $`(1-dq)G`$ and assigns $`qG`$ to each of its
$`d`$ earning ancestors. This attribution is divided once; it is not a separate
additional payment to the asset holder.

Equivalently, member $`i`$ has own income and gross referral overrides:

```math
A_{i,l}=\frac{Y_l}{P_l}(1-d_iq)p_{i,l},\qquad
O_{i,l}=\frac{Y_l}{P_l}qD_{i,l},
```

where $`D_{i,l}`$ includes every strict descendant's eligible balance. A
grandchild therefore funds the same commission fraction for its grandparent as
it does for its parent. A node can earn overrides without holding assets itself,
but every override is funded by actual descendant assets.

Core earnings are $`B_i=\sum_l(A_{i,l}+O_{i,l})`$. Across all members, they sum
to the nonempty lanes' assigned budget. If $`P_l=0`$, that lane's allocation is
unallocated and unissued; later deposits cannot claim it.

## Marketing weighted by escrowed credits

A parent chooses a discretionary fraction $`\mu_i\in[0,1]`$. Its source is gross
referral commissions $`O_i=\sum_l O_{i,l}`$, excluding own-asset income, fees,
escrow principal, and incoming family allocations.

Let $`s_c`$ be a child's issued-credit escrow with that parent and let
$`S_i=\sum_c s_c`$ over its immediate children. When $`S_i\gt 0`$:

```math
M_i=\mu_iO_i,\qquad G_{i,c}=M_i\frac{s_c}{S_i}.
```

If $`S_i=0`$, the parent retains the otherwise undistributable budget and
$`M_i=0`$. Its final retained entitlement is

```math
R_i=B_i-M_i+G_{parent(i),i},
```

with no incoming family allocation for a top-level seat. Children keep incoming
family allocations whole; those credits do not enter another marketing pool or
pay another commission.

There is one escrow weight and one marketing fraction per member relationship
across all lanes. A child needs no BTC or ETH holdings to receive a marketing
family allocation. It needs issued credits actually held in the corresponding
escrow. That principal remains withdrawable after settling earlier income and
cannot back another family simultaneously.

For a 30-credit pool, siblings escrowing 100 and 200 credits receive 10 and 20.
Splitting the first stake into aliases holding 40 and 60 yields 4 and 6, leaving
the owner's combined share unchanged. An empty account receives no family
allocation merely because it exists.

## Worked allocation across two assets

Take Alice at depth zero, her child Bob at depth one, and Bob's child Carol at
depth two. Let $`q=10\%`$, $`\mu_A=50\%`$, and $`\mu_B=25\%`$. Bob escrows 100
credits with Alice; Carol escrows 200 with Bob. These are already-issued
principal from earlier activity.

| Lane | Assigned credits | Alice's assets | Bob's assets | Carol's assets |
| ---- | ---------------: | -------------: | -----------: | -------------: |
| BTC  |              200 |            100 |          100 |              0 |
| ETH  |              100 |              0 |            0 |            100 |

The credits earned over this interval are:

| Credit flow              |   Alice |       Bob |    Carol |
| ------------------------ | ------: | --------: | -------: |
| Own-asset income         |     100 |        90 |       80 |
| Gross referral overrides |      20 |        10 |        0 |
| Marketing paid           |     -10 |      -2.5 |        0 |
| Marketing received       |       0 |        10 |      2.5 |
| **Net entitlement**      | **110** | **107.5** | **82.5** |

The total is 300. Bob receives part of Alice's commission attributed through the
ETH lane even though he holds no ETH. The 300 credits in family escrow remain
principal and do not enter either asset denominator or the new emission budget.

## Changes, incentives, and collection

Asset changes settle the affected lane and ancestor path under the old balances.
Changing escrow or a parent's marketing fraction additionally requires a common
checkpoint of all that parent's contributing lanes. The new values then apply
prospectively; a late staker cannot claim earlier rewards. See
[Emission Accounting](Emission-Accounting).

Parents can change future marketing percentages and join fees. These are
discretionary offers, not permanent income rights. Ordinary admission fees
transfer issued credits to the parent; they neither count as the child's escrow
nor create another emission claim.

Family growth can increase a child's family credit allocation, but credit escrow
alone does not grow the parent's asset-based income. A new sibling staking only
credits dilutes existing shares; a nephew bringing assets can increase the
family's commission pool. Declining global emission, competing escrow, gauge
changes, and asset withdrawals still affect the credits allocated.

Collection mints only the member's net earned entitlement and debits that claim
once. Incoming family allocations may be explicitly collected and restaked, but
do not automatically compound. [Mutualization](Mutualization) describes the
family market; [Link Shopping](Link-Shopping) explains admission offers.

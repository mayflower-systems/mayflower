<!--
title: Link Shopping
summary:
  Comparing permanent referral positions, credit admission prices, and changing
  family marketing offers before choosing a parent.
tags:
  - referral
  - marketplace
-->

A referral link offers a permanent position under a particular parent in the
[Referral Network](Referral-Network). Links can differ in depth, admission price,
existing downline income, competing credit escrow, and the parent's current
marketing policy. The position is permanent; its commercial attractiveness can
change.

## Compare the position and the offer

| Consideration | What it changes |
| --- | --- |
| Depth $d$ | The own-asset retention fraction $1-dq$ |
| Remaining depth $H-d$ | How many generations can grow beneath the new node |
| Join fee | Credits paid to the parent to enter |
| Fee increment | The quoted price after subsequent successful joins |
| Marketing fraction $\mu$ | How much of the parent's gross override income is currently shared |
| Parent's gross override income | The size of the credit-funded opportunity across asset lanes |
| Competing child escrow | How much of that marketing budget a given credit stake receives |
| Expected business behavior | Future recruitment, participation, prices, and discretionary marketing changes |

Shallower positions retain more of their own asset attribution and have more
recruiting room. A node at depth $H$ cannot recruit, but can earn from its own
assets and stake credits in its parent's family pool.

## Ordinary admission is priced in credits

An ordinary parent posts a current join fee and a nonnegative increment. A
successful join pays the current fee to the parent, then increases the quote by
the increment. With a 10-credit fee and a 2-credit increment, successive prices
are 10, 12, and 14 if the parent makes no intervening change.

The parent can replace both values for future joins. The joining transaction
can specify a maximum accepted fee, so a changed quote cannot charge more than
the payer authorized. The payment does not buy an immutable marketing rate.

Zero fee and zero increment are valid. A creditless newcomer can use an available
free link, deposit eligible assets, earn credits, and then escrow them. Someone
seeking a paid link must obtain credits first. A free entry does not later move
that node to a different parent.

The fee is separate from family escrow. Paying 20 credits to join and staking 80
credits requires 100 issued credits; only the 80 still owned by the child count
as its family stake. SOL transaction and account costs remain separate.

## Top-level admission is an auction

Scarce depth-zero seats are allocated through the protocol auction, not an
ordinary parent offer. The winning credit payment is burned. The logical
protocol root earns nothing and receives no commission or family stake.

The auction format and initial credit inventory are separate protocol
responsibilities. A public referral link cannot create another top-level seat.

## Estimate the current family income

Use the parent's offered budget $B=\mu O$, where $O$ is its current gross
override income in credits per day. If other children have $S$ credits escrowed,
adding your $s>0$ credits gives the current rate, assuming the source income is
otherwise unchanged:

$$\text{your gift rate}=B\frac{s}{S+s}.$$

The denominator must include the new stake. If $S=0$, your stake activates the
offered budget for future accrual; it does not claim the parent's earlier
earnings. For example:

| Family | Offered budget per day | Other child escrow | Your added escrow | Your current gift rate (credits/day) |
| --- | ---: | ---: | ---: | ---: |
| A | 30 | 200 | 100 | 10 |
| B | 20 | 50 | 100 | $13\frac{1}{3}$ |

The smaller pool can offer the better current rate because fewer credits compete
for it. If joining also changes asset participation, the preview must include
that change to the parent's income and global allocation.

These are current rates expressed per day, not promises of that amount over the
next day. Actual receipts integrate the declining emission schedule and changing
family state. Parents can change $\mu$, other children can
stake or withdraw, assets can move, and lane allocations can change. A popular
sibling's future recruits may enlarge the pool, but that expectation is not a
guaranteed return.

Escrowing credits does not require recruiting or depositing assets. A recipient
may simply compete for the parent's subsidy. The displayed offer is a current
quote, not a contract fixing future marketing payments; the parent may set
$\mu$ to zero after admission.

## Permanent edges, flexible balances

Joining fixes a node's parent and depth. It does not freeze its assets or family
escrow. Those balances can change frequently, with earlier earnings settled under
the previous state. A person may own multiple nodes and allocate actual capital
among their available offers; their existing connections stay in place.

A person can join another family through a new node and move withdrawable assets
and credit principal there. The old admission fee is not refunded, and the old
node's descendants and referral position do not follow. Accrued earnings remain
claimable through the old node. This permits capital to leave an unattractive
offer without making an existing subtree portable.

The target rules do not require identifying which children a parent controls.
Those accounts need real escrow to receive marketing gifts. A parent may also
reduce its marketing budget directly, so reputation and expected future business
are relevant to the offer's value, not substitutes for a promised permanent rate.

See [Mutualization](Mutualization) for the pool mechanics and
[Rationale](Referral-Network-Rationale) for the economic limits.

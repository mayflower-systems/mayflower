<!--
title: Revenue Clearing Time Weight
summary:
  How the clearing auction's entry weight, withdrawal penalty, and
  last-in-first-out deposit lots affect committed tokens and reward shares.
tags:
  - revenue-clearing
  - tokenomics
-->

In a [clearing auction](Revenue-Clearing), a participant commits clearing
tokens for shares of an epoch's reward pot. Entry time determines the weight of
a deposit; withdrawal time determines the penalty on the requested principal.
Each deposit has its own lot. A withdrawal removes principal and shares from
the newest lots first while preserving the shares of untouched older lots.

## Entry weight

Let $T$ be the epoch duration and $t$ the time since its start. For a deposit
before the deadline, the ideal entry weight is

$$w(t)=1+b\left(1-\frac{t}{T}\right),\qquad 0\le b\le2.$$

A deposit of $a$ clearing-token units earns $a\,w(t)$ shares in ideal
arithmetic. Shares are fixed when the deposit is made; they do not decay with
time. With the reference $b=1$:

- At the start, a token earns **2 shares**.
- Halfway through, it earns **1.5 shares**.
- Near the deadline, it earns **nearly 1 share**; the deadline itself is closed
  to deposits.

At $b=0$, weight is always 1. Increasing $b$ increases the early-entry
advantage, up to a starting weight of 3 at $b=2$. The slope sets the tradeoff
between committing early and waiting to observe more of the auction. Fixed-point
rounding can make nearby entry times tie; see the
[formal specification](Revenue-Clearing-Formal-Specification).

## Withdrawal penalty

For a withdrawal at elapsed time $t$, the ideal penalty fraction is

$$p(t)=\left(\frac{t}{T}\right)^k,\qquad k\in\{1,2,3,4\}.$$

Withdrawing $x$ units returns $x(1-p(t))$ in ideal arithmetic and forfeits
$xp(t)$, burned during the withdrawal. The penalty uses time since the
**epoch start**, not the age of the deposit. With the reference $k=2$ and a
seven-day epoch:

- After one day, the ideal penalty is about **2%**.
- Halfway through, it is **25%**.
- After five days, it is about **51%**.
- After six days, it is about **73%**.
- At the deadline, its curve reaches **100%**; withdrawals have already stopped.

A larger exponent reduces the ideal penalty earlier in the epoch and
concentrates it closer to the deadline. The penalty makes late reversal of a
commitment costly. A withdrawal also destroys the corresponding shares. The
forfeited principal does not enter the reward pot, while destroying shares
increases the remaining positions' fraction of a fixed pot.

## Lots and withdrawal order

Every deposit creates a lot with its own principal and shares. A position's lots
are last-in-first-out: one withdrawal can consume the newest surviving lot and
continue through older lots until it has removed the requested principal.
Since entry weight is non-increasing, the newest lot has the lowest weight or
ties with an older lot.

In ideal arithmetic, withdrawing a fraction of a lot's remaining principal
removes the same fraction of its remaining shares. Thus the newest lot removes
the fewest shares for a given amount of principal, preserving older,
higher-weight lots. Actual accounting rounds partial share removal up and the
returned principal down. The refund is calculated once for the entire requested
amount, even when it spans lots. Splitting the request into separate actions can
reduce the refund: at a 25% penalty, one two-unit withdrawal returns one unit,
while two one-unit withdrawals return zero each.

For example, suppose a position holds an older lot of 100 principal and 200
shares, then a newer lot of 100 principal and 150 shares. Withdrawing 150
principal removes all 150 shares of the newer lot and 100 shares of the older
lot. The position keeps 50 principal and 100 shares. At a 25% penalty, the
request returns $\lfloor150\times0.75\rfloor=112$ principal and burns 38.

## Worked example: withdrawal through settlement

Suppose an epoch uses $b=1$ and $k=2$. The quantities below are whole token base
units, and the selected times and amounts make the intermediate calculations
exact.

1. At the start, Alice deposits 100 clearing units at weight 2 and receives 200
   shares.
2. Halfway through, Bob deposits 100 clearing units at weight 1.5 and receives
   150 shares. He then withdraws 40 units from that lot at the same halfway
   point. The 25% penalty forfeits 10 units and returns 30. His withdrawal
   removes 60 shares, leaving 60 units of principal and 90 shares.
3. A funder supplies 90 reward units, all of which enter the pot. Clearing
   charges no funding fee. Assume no further activity.
4. Finalization burns Alice's 100 and Bob's remaining 60 clearing units. The
   frozen pot is 90 reward units and total surviving shares are $200+90=290$.
5. Alice claims $\lfloor90\times200/290\rfloor=62$ reward units; Bob claims
   $\lfloor90\times90/290\rfloor=27$. After both claims, the remaining reward
   unit stays in escrow.

The 200 clearing units deposited are accounted for as 30 returned to Bob, 10
burned on withdrawal, and 160 burned at finalization. The 90 reward units are
accounted for as 89 in participant payouts and one unit of remainder. Alice's
fraction of the pot increased when Bob removed shares, but
his 10-unit clearing-token penalty did not add reward to the pot. Clearing and
reward units may be different assets, so their amounts are not added together.

If Bob had another, older lot, withdrawing from his newest lot would leave the
older lot unchanged. His 90 surviving shares receive their fraction at
settlement; the 60 shares removed by withdrawal cannot claim. If he had fully
withdrawn every lot, he would have no surviving shares.

## Settlement

At the deadline, deposits, withdrawals, and funding stop. Finalization burns
remaining principal and freezes the reward $R$ and surviving shares $S$. For
$S>0$, a position with $S_i>0$ shares claims

$$P_i=\left\lfloor\frac{R S_i}{S}\right\rfloor_{\text{reward unit}}.$$

Each position claims once, without a claim deadline. Claims use the same frozen
numerator and denominator, so their order does not change the payouts. After all
shares have claimed, the remainder stays in escrow. If no shares survive, no
position can claim and the entire pot stays in escrow. The current mechanism
has no epoch-closure or remainder-sweep action.

The ratio $R/S$ is reward per weighted share. To understand a particular
participant's reward per clearing token, account for that participant's entry
weights and withdrawals. The [rationale](Revenue-Clearing-Rationale) explains
why the model makes these time-dependent choices.

<!--
title: Long-Term Economics of Credits
summary:
  Declining emission, competing family offers, channel pledges, admission, and
  clearing create distinct uses for scarce issued credits.
tags:
  - tokenomics
  - economics
-->

Mayflower credits connect [emission allocation](Credit-Emissions), referral
admission, [family marketing](Mutualization), and
[revenue clearing](Revenue-Clearing). This rationale explains their economic
relationship. The
[referral specification](Referral-Network-Formal-Specification),
[emission accounting](Emission-Accounting), and
[clearing specification](Revenue-Clearing-Formal-Specification) define the
actual allocation and settlement rules. The economic examples below are
conditional comparisons, not additional algorithm rules or return guarantees.

## Earning and directing credits

The emitter supplies a credit rate $E(t)$ that is planned to decrease
continuously. Its integral determines the scheduled budget over an interval.
Channel pledges direct that budget's relative distribution among asset lanes;
they do not choose the emitter's decay curve. Within a lane, actual eligible
assets determine gross attribution, and fixed commissions divide it between each
holder and its earning ancestors.

Adding assets changes proportional claims on the same budget. Recruiting someone
with assets can benefit an ancestor, but neither an additional address nor a new
family creates another emission stream. Empty or otherwise unavailable
allocations remain explicitly unallocated rather than becoming later backpay.

## Competing family offers

Parents compete using credit-denominated join fees and a discretionary marketing
percentage. A parent's marketing budget comes only from its gross referral
commissions across lanes. Its immediate children divide that budget according to
their escrowed credits, regardless of whether they themselves hold eligible
assets. A sibling's descendants can therefore contribute to a shared family pool
without adding to that family's escrow denominator.

The parent may change its percentage for future accrual, and new credit stakes
dilute existing shares. A child's principal remains withdrawable after
settlement. The permanent parent connection stays in place even if the owner
moves capital through another node. Joining purchases membership under the
quoted fee; it does not purchase a permanent marketing rate or guaranteed yield.

Escrow uses scarce credits rather than account count: splitting the same credits
across sibling aliases preserves the owner's combined marketing share in ideal
arithmetic. The fixed-commission core similarly does not reward splitting fixed
assets within a connected owned region with unchanged outside ancestry. These
are limited accounting properties, not identity detection or universal
protection against pooling, coalitions, or shopping among different offers.

Parents may reclaim their own discretionary budget through controlled accounts,
just as they may reduce their marketing percentage. Self-paid join fees are
internal transfers. Market demand may influence these choices, but the protocol
does not guarantee that a parent's marketing expense earns a profit.

Family allocations do not automatically compound. If everyone in one pool
voluntarily reinvests proportionally, that compounding alone leaves their
relative shares unchanged. Different outside earnings, purchases, withdrawals,
and allocation choices can change those shares. Neither compounding nor family
growth guarantees an increasing credit income: global dilution, gauge
allocations, competing escrow, and changing offers all affect it.

## Choosing a use for issued credits

The reference integration keeps channel pledges, family escrow, clearing
commitments, and auction bids in exclusive custody. One credit cannot back those
commitments simultaneously. A channel pledge directs emission; family escrow
earns a share of an offered marketing pool. Neither action itself burns credits.

Ordinary join fees transfer credits to a parent. Scarce top-level seats require
authorized auction admission whose winning credit payment is burned. These are
different uses, and neither counts as the newcomer's family escrow. A free link
can let someone earn initial credits from assets, but availability is a market
choice. The first credit-paid seat auction requires an explicitly documented
existing bootstrap supply; the referral system does not mint its own payment.

## Spending credits on revenue

Clearing participants commit credits to claim shares of a funded reward pot.
Remaining principal is burned at finalization; an earlier withdrawal burns its
applicable penalty immediately. The reward a participant receives depends on the
funded pot, competing shares, entry time, withdrawals, and rounding.
Finalization still burns committed principal if the pot is zero.

The auction gives credits a concrete use, but it does not establish a fixed
redemption price. A sole participant can receive the whole positive pot with a
small accepted bid. Independent competition determines how many credits are
committed, and additional addresses do not establish independent competition.
Early-entry weights offer additional shares for earlier commitment. Whether that
offsets commitment costs depends on participants' expectations; the weights do
not eliminate information differences or advantages from transaction ordering.

Clearing and emission remain separate. Spending credits in clearing does not
automatically change the emission rate, and new emission does not automatically
fund a clearing pot. Clearing's existing entry, withdrawal, and forfeiture rules
are unchanged by referral mutualization.

## Revenue growth and credit dilution

Committing credits to a clearing epoch creates weighted accounting shares in its
funded reward pot. Holding credits alone creates no such position. More eligible
assets can dilute a member's credit share even when useful activity grows the
reward pot. The two effects must be measured separately.

For a restricted benchmark, assume one asset lane, unchanged eligible balances
throughout the earning interval, and no family marketing. All and only the
credits earned in that interval are collected and committed to one pot at the
same entry weight, and remain committed through finalization. Ignore rounding
and old credit inventories. A member with core numerator $K=(1-dq)p+qD$ and
global eligible balance $P$ then receives $K/P$ of a pot containing $R$
reward-asset units:

$$\text{reward payout}=R\frac{K}{P}.$$

If, additionally, genuine funded revenue scales as $R=rP$ for constant $r$,
recruiting $x$ additional eligible units into that member's downline changes its
payout from $rK$ to $r(K+qx)$: a gain of $rqx$. Its credit share can still fall
because the denominator grows. If the reward pot instead stays fixed, that
revenue-growth benefit does not exist.

These assumptions are not properties of the auction. Real outcomes also depend
on credit inventories, other uses of credits, entry times, and bidding choices.
Depositing an asset does not by itself prove that it generates proportional
revenue. The benchmark only shows why credit dilution and reward-asset income
are different questions.

## Revenue must come from activity

The intended source is AVM protocol fees: each fee-bearing market action
transfers its protocol allocation directly into a clearing pot denominated in
that market's reserve asset. Trading and reserve-access activity can therefore
fund rewards without requiring new referral admissions. The amount depends on
actual activity and configured fee rates, not merely the number of accounts or
the quantity of eligible assets.

Burning credits is payment for a funded pot; it does not create that pot's
revenue. Family marketing likewise reallocates commissions rather than creating
new revenue or a second emission stream. Reinvesting earned credits changes who
receives later allocations, subject to the same budget constraints.

A durable model needs useful activity that continues funding pots even during a
long period with no new members. Admission payments, newly deposited user
capital, and reserve backing must not be mistaken for freely distributable
revenue. Whether genuine fees or other earnings suffice depends on the activity
and funding sources of the deployment. The allocation algorithms alone cannot
establish it.

## Supply and value

Collecting net earned rewards issues credits once; uncollected entitlement is
not yet usable principal. Fees and escrow transfers leave issued supply
unchanged. Auction payment burns and clearing burns reduce supply without
replenishing the emission budget. Locked principal remains part of issued supply
until an actual burn occurs.

The cumulative scheduled emission continues increasing while $E(t)$ is positive,
even as its rate declines. This cumulative amount is distinct from issued
supply: uncollected entitlements and unallocated budget are not issued tokens.
The intended scarcity effect is a progressively smaller flow of new credits; it
does not imply a progressively smaller outstanding stock.

The issued supply balance is documented bootstrap supply plus collected emission
claims minus burns. It may rise or fall depending on those flows. Neither the
declining emission rate nor the allocation rules guarantee a particular market
price, circulating supply, or return from any use of credits.

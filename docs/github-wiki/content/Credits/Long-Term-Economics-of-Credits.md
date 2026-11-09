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
admission, [family marketing](Mutualization), and [revenue clearing](Revenue-Clearing).
This page explains the
[referral specification](https://github.com/mayflower-systems/mayflower/blob/a1f87bfa8469630444626ddbc11536512c78bb16/referral/rust/mayf-referral/SPEC.md),
whose allocation rules are implemented by the referral kernel. Custody and
the emission schedule belong to its host integration.

## Earning and directing credits

The emitter supplies a credit rate `E(t)` that is planned to decrease
continuously. Its integral determines the scheduled budget over an interval.
Channel pledges direct that budget's relative distribution among asset lanes;
they do not choose the emitter's decay curve. Within a lane, actual eligible
assets determine gross attribution, and fixed commissions divide it between
each holder and its earning ancestors.

Adding assets changes proportional claims on the same budget. Recruiting someone
with assets can benefit an ancestor, but neither an additional address nor a new
family creates another emission stream. Empty or otherwise unavailable
allocations remain explicitly unallocated rather than becoming later backpay.

## Competing family offers

Parents compete using credit-denominated join fees and a discretionary marketing
percentage. A parent's marketing budget comes only from its gross referral
commissions across lanes. Its immediate children divide that budget according to
their escrowed credits, regardless of whether they themselves hold eligible
assets. A sibling's descendants can therefore contribute to a shared family
pool without adding to that family's escrow denominator.

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

Gifts do not automatically compound. If everyone in one pool voluntarily
reinvests proportionally, that compounding alone leaves their relative shares
unchanged. Different outside earnings, purchases, withdrawals, and allocation
choices can change those shares. Neither compounding nor family growth
guarantees an increasing credit income: global dilution, gauge allocations,
competing escrow, and changing offers all affect it.

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
Remaining principal is irreversibly forfeited at finalization, while an earlier
withdrawal forfeits its applicable penalty. The reward a participant receives
depends on the pot, competing shares, entry time, withdrawals, and rounding.

Clearing and emission remain separate. Spending credits in clearing does not
automatically change the emission rate, and new emission does not automatically
fund a clearing pot. Clearing's existing entry, withdrawal, and forfeiture rules
are unchanged by referral mutualization.

## Supply and value

Collecting net earned rewards issues credits once; uncollected entitlement is
not yet usable principal. Fees and escrow transfers leave issued supply
unchanged. Auction payment burns and actual burns of forfeited clearing credits
reduce supply without replenishing the emission budget. Clearing forfeitures
can remove spendable credits before their subsequent token burn.

The cumulative scheduled emission continues increasing while `E(t)` is positive,
even as its rate declines. This cumulative amount is distinct from issued
supply: uncollected entitlements and unallocated budget are not issued tokens.

The issued supply balance is documented bootstrap supply plus collected emission
claims minus burns. It may rise or fall depending on those flows. Neither the
declining emission rate nor the allocation rules guarantee a particular market
price, circulating supply, or return from any use of credits.

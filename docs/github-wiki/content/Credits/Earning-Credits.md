<!--
title: Earning Credits
summary:
  Members earn newly issued credits when eligible collateral and referral
  relationships determine retained rewards from a channel's emission.
tags:
  - tokenomics
  - emission
-->

Participants earn [Mayflower credits](Mayflower-Credits) through the
[emission system](Credit-Emissions) and the
[Referral Network](Referral-Network). The emission schedule determines a common
reward stream; channel pledges divide it among market channels; each channel's
eligible points and referral relationships determine who earns its allocation.

## Participation and referral rewards

In a channel, eligible members' deposited collateral supplies points. A member's
own points are the starting basis for its share of the channel's allocation.
Each source's attribution is divided between the source and its upline by a
fixed override rate. Members may voluntarily share part of their available
earnings with direct children, who can share receipts onward.

The final amount a member retains after overrides and voluntary sharing is its
reward. A member can earn from a productive downline even with no deposit of
its own. Referral distribution divides the channel's allocation; it does not
create a second reward from the same emission.

Membership makes eligible collateral count toward emission points. Points begin
earning prospectively when eligible; joining does not create rewards for an
earlier interval. AVM use and channel pledges do not require referral
membership.

## From entitlement to supply

The emission schedule records an accruing entitlement; elapsed time alone does
not issue credits. A member collects its retained reward entitlement, and
collection issues the corresponding whole credit base units after Reserve has
taken over the mint authority. Fractional reward entitlement remains accounted
for. Any bootstrap supply that existed before that handoff is separate from
scheduled rewards.

See [Participation and Referral Rewards](Participation-and-Referral-Rewards) for
the allocation and examples, and
[Accrual, Collection, and Conservation](Emission-Accounting) for how
reward history and issuance are accounted.

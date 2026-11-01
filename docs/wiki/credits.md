---
title: Mayflower Credits
summary:
  Mayflower credits are earned through referral-based emissions, pledged to
  weight channels, and spent in revenue clearing.
tags:
  - tokenomics
---

Mayflower credits connect participation, emission allocation, and revenue
clearing. Members earn credits through
[[credits--earning|referral-based emissions]]. Holders can
[[credits--channel-weighting|pledge credits to channels]] to direct future
emission, or [[credits--revenue-clearing|commit credits to revenue clearing]] to
claim a share of a funded reward pot.

Credits have three roles in the protocol:

1. **Earn:** Eligible points and referral relationships determine a member's
   share of a channel's emission. Collecting retained rewards issues credits.
2. **Direct:** A holder pledges already-issued credits to a channel. The pledge
   contributes weight to that channel's share of future emission and can later
   be reduced.
3. **Spend:** A participant commits credits to a clearing auction. Remaining
   principal is irreversibly forfeited at finalization; earlier withdrawals
   return their unpenalized portion and forfeit the rest.

The protocol calculates allocations and clearing claims from its published
rules. Holders control their own channel pledges and can reduce them under those
rules; pledged credits cannot simultaneously be committed to clearing. Channel
pledges and clearing participation do not themselves alter the emission
schedule.

## Notes

- [[credits--earning|Earning Credits]] — How participation and the referral
  network allocate new emission.
- [[credits--channel-weighting|Credit Channel Weighting]] — How pledges direct
  emission among channels.
- [[credits--revenue-clearing|Credits and Revenue Clearing]] — How credits are
  spent to claim funded rewards.
- [[credits--long-term-economics|Long-Term Economics of Credits]] — How earning,
  directing, and spending interact.

[[credits-emissions|Credit Emissions]], the
[[referral-network|Referral Network]], and [[revenue-clearing|Revenue Clearing]]
provide the specifications for their respective mechanisms.

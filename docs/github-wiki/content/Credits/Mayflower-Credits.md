<!--
title: Mayflower Credits
summary:
  Mayflower credits are earned through referral-based emissions, pledged to
  weight channels, and spent in revenue clearing.
tags:
  - tokenomics
-->

Mayflower credits connect participation, emission allocation, and revenue
clearing. Members earn credits through
[referral-based emissions](Earning-Credits). Holders can
[pledge credits to channels](Credit-Channel-Weighting) to direct future
emission, or [commit credits to revenue clearing](Credits-and-Revenue-Clearing) to
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

- [Earning Credits](Earning-Credits) — How participation and the referral
  network allocate new emission.
- [Credit Channel Weighting](Credit-Channel-Weighting) — How pledges direct
  emission among channels.
- [Credits and Revenue Clearing](Credits-and-Revenue-Clearing) — How credits are
  spent to claim funded rewards.
- [Long-Term Economics of Credits](Long-Term-Economics-of-Credits) — How earning,
  directing, and spending interact.

[Credit Emissions](Credit-Emissions), the
[Referral Network](Referral-Network), and [Revenue Clearing](Revenue-Clearing)
provide the specifications for their respective mechanisms.

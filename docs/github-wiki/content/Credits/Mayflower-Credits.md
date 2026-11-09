<!--
title: Mayflower Credits
summary:
  Credits connect referral rewards, admission, family mutualization, channel
  weighting, and revenue clearing through distinct custody and payment rules.
tags:
  - tokenomics
-->

Mayflower credits connect [referral rewards](Earning-Credits), admission,
[family mutualization](Mutualization), [channel weighting](Credit-Channel-Weighting),
and [revenue clearing](Credits-and-Revenue-Clearing).

The referral and custody rules here describe the
[implementation specification](https://github.com/mayflower-systems/mayflower/blob/a1f87bfa8469630444626ddbc11536512c78bb16/referral/rust/mayf-referral/SPEC.md).
The pure referral kernel implements the allocation and settlement rules. The
surrounding programs must enforce token custody, auction awards, and authorization;
the specification does not imply that this integration is deployed.

## Earning and using credits

| Operation | What happens to credits |
| --- | --- |
| Earn referral rewards | Eligible assets earn a share of scheduled global emission; fixed commissions reward their ancestors. Collection issues the member's net earned credits once. |
| Join an ordinary parent | The newcomer transfers the parent's quoted credit fee to that parent. The fee is neither burned nor counted as the child's family escrow. |
| Acquire a top-level seat | An authorized auction allocates a scarce seat. Its winning credit payment is burned; refundable bids are handled separately by the auction. |
| Escrow with a family | A child commits issued credits to weight its share of its parent's discretionary marketing budget. Principal remains withdrawable after settlement. |
| Pledge to a channel | Issued credits weight that channel's share of future emission. A holder can later reduce the pledge. |
| Commit to clearing | Issued credits acquire shares of a funded reward pot under the clearing rules, including withdrawal penalties and final forfeiture. |

A family's marketing budget comes only from its parent's gross referral
commissions across asset lanes. It excludes the parent's own-asset yield, fees,
escrow principal, and incoming gifts. A child can receive a marketing gift with
no eligible assets of its own if it has escrowed credits. Gifts are kept rather
than shared onward, and they do not automatically compound.

## Custody and supply

The reference integration uses exclusive custody. The same issued credits cannot
simultaneously back family escrow, channel pledges, clearing commitments, or
auction bids. One family escrow balance does weight all asset lanes feeding that
family's common pool. Using family escrow for governance as well would require a
separate integration rule.

Uncollected rewards are entitlements, not issued tokens available for fees,
pledges, bids, or escrow. Collection debits the entitlement and issues the
corresponding credits. Transfers and escrow movements leave supply unchanged;
burns reduce supply. Burning credits does not authorize replacement emission.

The global emission rate `E(t)` is planned to decrease continuously. The emitter
determines that rate; channel weights and family marketing divide the scheduled
budget for each interval without increasing it. A declining rate still adds to
cumulative scheduled emission while it remains positive. Net issued supply also
depends on collection and burns, so it need not move in the same direction.

The first credit-paid seat auction requires documented, pre-existing bootstrap
credits. The referral system does not mint credits to pay its own auction. Once
seats exist, an available zero-fee ordinary link lets a creditless newcomer join,
earn credits from assets, and then escrow them. Free offers are permitted, not
guaranteed.

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

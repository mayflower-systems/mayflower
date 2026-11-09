<!--
title: Mayflower Credits
summary:
  A common bidding unit connects asset-weighted participation to AVM fee pots,
  without converting those fees into a common reserve asset or dollar value.
tags:
  - tokenomics
-->

Mayflower credits are the common bidding unit for
[revenue clearing](Revenue-Clearing). Participants earn them through
asset-weighted [referral rewards](Earning-Credits) and spend them for shares of
fee pots denominated in different reserve assets. Credits also direct future
emission through [channel weighting](Credit-Channel-Weighting) and support
referral admission and [family mutualization](Mutualization).

## From market activity to reward allocation

The intended AVM integration connects the components as follows:

1. **Market actions assess fees in the base reserve asset.** Issuance,
   redemption, cash advances, issuance at the floor, and redemption at the floor
   use their configured fee rates. The collected fee is split into protocol and
   market-owner allocations.
2. **The protocol allocation funds clearing directly.** The AVM action transfers
   that allocation into the matching reserve-asset clearing pot in the same
   transaction. Fees remain in their original asset; there is no intermediate
   liquidation desk or conversion into dollars or a common token.
3. **Asset participation earns credits.** The emission schedule supplies a
   decreasing flow of new credits. Gauges divide that budget among asset lanes;
   the referral network allocates each lane's budget using eligible balances,
   fixed upline commissions, and optional family allocations. Collection issues
   the earned credits.
4. **Participants spend credits on fee pots.** They choose which pots to enter
   and how many credits to commit. Surviving weighted shares determine their
   portions of each pot, and remaining committed credits are burned at
   finalization.

This connects participation rewards to actual AVM fees without making the
protocol determine the dollar value of either the fees or the eligible assets.
The [clearing rationale](Revenue-Clearing-Rationale) explains why a common
credit unit is useful and which valuation decisions remain with participants.

The [referral specification](Referral-Network-Formal-Specification) defines
admission and reward allocation. [Emission Accounting](Emission-Accounting)
defines settlement, collection, and custody. These are distinct operations:
earning a claim, holding an issued token, and committing that token elsewhere
must not be counted as the same economic event.

## Earning and using credits

| Operation                | What happens to credits                                                                                                                                         |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Earn referral rewards    | Eligible assets earn a share of scheduled global emission; fixed commissions reward their ancestors. Collection issues the member's net earned credits once.    |
| Join an ordinary parent  | The newcomer transfers the parent's quoted credit fee to that parent. The fee is neither burned nor counted as the child's family escrow.                       |
| Acquire a top-level seat | An authorized auction allocates a scarce seat. Its winning credit payment is burned; refundable bids are handled separately by the auction.                     |
| Escrow with a family     | A child commits issued credits to weight its share of its parent's discretionary marketing budget. Principal remains withdrawable after settlement.             |
| Pledge to a channel      | Issued credits weight that channel's share of future emission. A holder can later reduce the pledge.                                                            |
| Commit to clearing       | Issued credits acquire shares of a funded reward pot under the clearing rules, including burns of withdrawal penalties and remaining principal at finalization. |

A family's marketing budget comes only from its parent's gross referral
commissions across asset lanes. It excludes credits attributed to the parent's
own assets, fees, escrow principal, and incoming family allocations. A child can
receive a family allocation with no eligible assets of its own if it has
escrowed credits. Family allocations are kept rather than shared onward, and
they do not automatically compound.

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

The global emission rate $E(t)$ is planned to decrease continuously. The emitter
determines that rate; channel weights and family marketing divide the scheduled
budget for each interval without increasing it. A declining rate still adds to
cumulative scheduled emission while it remains positive. Net issued supply also
depends on collection and burns, so it need not move in the same direction.

The first credit-paid seat auction requires documented, pre-existing bootstrap
credits. The referral system does not mint credits to pay its own auction. Once
seats exist, an available zero-fee ordinary link lets a creditless newcomer
join, earn credits from assets, and then escrow them. Free offers are permitted,
not guaranteed.

## Notes

- [Earning Credits](Earning-Credits) — How participation and the referral
  network allocate new emission.
- [Credit Channel Weighting](Credit-Channel-Weighting) — How pledges direct
  emission among channels.
- [Credits and Revenue Clearing](Credits-and-Revenue-Clearing) — How credits are
  spent to claim funded rewards.
- [Long-Term Economics of Credits](Long-Term-Economics-of-Credits) — How
  earning, directing, and spending interact.

[Credit Emissions](Credit-Emissions), the [Referral Network](Referral-Network),
and [Revenue Clearing](Revenue-Clearing) provide the specifications for their
respective mechanisms.

<!--
title: Protocol Scope
summary:
  Protocol instances, permissionless execution, authorization, and the accounting
  terms used throughout the wiki.
tags:
  - mayflower
  - architecture
-->

Mayflower is a set of open-source algorithms for reserve-backed markets, credit
allocation, and funded reward auctions. This wiki describes their state,
transitions, and intended composition. Developers can implement, deploy, or
extend these mechanisms under the software license.

## Protocol and deployment

A **protocol instance** is a particular set of configured state and asset
bindings. In the described composition, one instance has one scheduled credit
budget and referral network. **Global emission** means the budget across that
instance's lanes. Independent deployments do not automatically share a budget,
tree, reserve, or credit mint.

The **integrating program**, also called the host, binds state to accounts,
verifies authorization and time, and settles token movements with state changes.
It may itself be an on-chain program. A library calculation or wiki formula does
not establish the custody, configuration, or upgrade controls of a live
deployment.

## Permissionless execution and authorization

A permissionless public operation accepts a valid request without discretionary
approval from an operator. Ownership checks still protect a participant's
position, and an operation must satisfy its balance, timing, and other rules.
Open-source availability and permissionless execution are distinct properties.

For example, [clearing finalization](Revenue-Clearing-Formal-Specification) can
be invoked by any caller after the deadline. Moving a participant's tokens or
claiming its position requires that position's authorization. Configuring an
auction channel is a separate authority-controlled action.

| Scope               | Control described by these algorithms                                                                                                                                                      |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| AVM market          | The integrating program authenticates fee configuration, floor-raise authorization, and optional floor-issuance rights. Every accepted transition must satisfy the mathematical checks.    |
| Referral member     | The integration authenticates eligible asset balances and authorizes custody movements. A member controls its credit escrow; a parent controls its future join fee and marketing fraction. |
| Root-seat admission | Activation consumes a valid auction award once; the auction defines allocation of the scarce seats.                                                                                        |
| Emission            | The integration verifies cumulative emission against the configured schedule and clock; gauge weights divide that budget.                                                                  |
| Clearing channel    | Its configuration authority sets terms for future epochs. Existing epochs retain their terms.                                                                                              |

An authority is an authorization role; a deployment may assign it to an account
or another program. Each deployment must state which actions are public, which
require a role, and which configuration or upgrade powers remain. The algorithms
do not prescribe one organization to exercise those powers.

## Accounting vocabulary

- **Emission channel or asset lane:** a balance denominator and allocation
  history for one eligible asset. A **clearing channel** instead configures a
  series of reward auctions. The two are not automatically the same object.
- **Credits:** issued token units used for fees, escrow, weighting, and
  clearing. Collection issues accrued emission claims; bootstrap supply is
  specified separately. Merely holding credits creates no clearing-position
  entitlement; committing them to an epoch creates its weighted shares.
- **Share or entitlement:** a quantity used by the specified allocation or claim
  operation. A clearing share applies to one funded epoch pot.
- **Family allocation:** credits reassigned from a parent's referral commissions
  to its immediate children under the marketing and escrow rules.
- **Escrow:** designated token custody with specified deposit, withdrawal, burn,
  and claim transitions. Which program controls it is a deployment fact.
- **Protocol fee and market-owner fee:** configured fee allocations and
  recipient roles. In the intended AVM integration, the protocol allocation goes
  directly to the matching reserve-asset clearing pot as part of the market
  transaction. A deployment binds the accounts and their controls.
- **Revenue clearing:** distribution of accepted funding, including the AVM's
  protocol-fee allocations. The algorithm also accepts other supplied assets; it
  does not establish a claim on future revenue or make market backing available
  for distribution.

These terms describe the accounting model. Legal classification depends on the
actual arrangement and applicable jurisdiction. The examples illustrate
specified inputs and outputs, rather than forecasts of participant earnings.

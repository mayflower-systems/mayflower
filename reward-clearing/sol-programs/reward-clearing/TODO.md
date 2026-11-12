# Reward-clearing account constraints and settlement TODO

Updated 2026-11-10. RC-001 through RC-003 are addressed by explicit account
constraints and active runtime regressions. The remaining items are policy
decisions, not confirmed authorization exploits.

## Addressed findings

- [x] **RC-001 — Withdrawal cannot target its own escrow.**
      [`bidder_position_withdraw.rs`](prog/src/instructions/bidder_position_withdraw.rs)
      rejects an aliased refund destination with `WithdrawalDestinationIsEscrow`
      before burning or transferring tokens. Previously, withdrawing 40 from a
      100-credit position halfway through the epoch could burn 20, return nothing,
      and leave 80 escrow tokens backing only 60 recorded credits. The regression
      checks atomic rejection both with and without a penalty.
- [x] **RC-002 — Claim cannot target its own escrow.**
      [`bidder_position_claim.rs`](prog/src/instructions/bidder_position_claim.rs)
      rejects an aliased reward destination with `ClaimDestinationIsEscrow` before
      payout, state updates or position closure, including when the payout is zero.
      Previously, a claim could close the position and consume its 250-token payout
      entitlement while all 250 tokens remained in the reward escrow. The regression
      checks that rejection preserves the position, rent and custody.
- [x] **RC-003 — Clearing credits cannot use the native mint.**
      [`create_channel_group.rs`](prog/src/instructions/create_channel_group.rs)
      rejects wrapped SOL with `NativeClearingMint`. Previously, a native-credit
      group could accept deposits but could not perform penalized withdrawals or
      finalize because SPL does not burn native tokens. The regression checks
      rejection of standalone group creation and a packed organization/group
      transaction, including counters, allocations and rent. Native reward tokens
      remain permitted because reward settlement uses transfers.

The two escrow-alias findings consumed the signing bidder's own entitlement;
the audit did not demonstrate theft of another bidder's funds. The fixes reject
those transactions rather than allowing successful settlement without payout.

## Open policy decisions

- [ ] **RC-004 — Decide the mint freeze-authority policy.** Creation accepts mints
      with a freeze authority. That issuer can freeze custody accounts and block
      transfers or burns. Decide whether such assets are permitted with that trust
      assumption, or must be rejected at group/channel creation. This requires the
      mint's freeze authority and is not an outsider authorization bypass.
- [ ] **RC-005 — Define surplus custody handling.** Anyone can directly transfer
      tokens into a public escrow. Exact equality between physical custody and
      recorded balances is therefore not a universal invariant. Specify whether
      donations, rounding dust and other excess tokens remain locked or have an
      authorized recovery path. For live credit epochs and unclaimed reward pots,
      require sufficient backing; finalized credit totals are historical because
      those tokens have been burned. Unsolicited donations must not prevent ordinary
      settlement.

## Verification

The active [account-constraint regressions](litesvm/tests/account_constraints.rs)
use generated CPI instructions and assert explicit errors plus complete rollback
of supplied accounts. Run them with the rest of the suite:

```sh
cargo test --locked -p mayf-reward-clearing-sol-litesvm
```

Run `build.sh` after program changes to refresh the binary and generated clients
before testing. The suite also exercises signer and ownership checks, malformed
instructions, custody substitution, token authorization and later-CPI rollback.
These focused checks do not establish that every possible transaction is safe.

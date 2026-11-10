# Weighted emitter follow-up

## Preserve withdrawal availability when emission accounting overflows

Vote changes settle accrued emission before changing the gauge's weight. If a
gauge's accrued allocation exceeds `u64`, the pure transition rejects it and
`withdraw_vote` cannot refund the user's principal. Transaction rollback keeps
the accounts consistent, but subsequent withdrawals still encounter the same
overflow. Changing future decay cannot remove the historical allocation.

Confirmed against the built SBF program: create a zero-decay emitter with an
initial rate of `u64::MAX` raw token units per second, deposit one voting token,
advance the clock three seconds, then withdraw that token. The transaction fails
with `ArithmeticOverflow` (6004); the recorded deposit and escrow remain intact.
The runtime regression
`overflowing_emission_rejects_withdrawal_without_changing_custody` preserves this
reproduction in [`litesvm/tests/lifecycle.rs`](litesvm/tests/lifecycle.rs).

Choose an explicit policy before relying on unrestricted schedule configuration
for custody: a lifetime emission budget that halts accrual at its limit, or a
withdrawal recovery transition with defined treatment of unrepresentable
emission. Merely saturating counters or skipping settlement would risk changing
other gauges' allocation or authorizing repeated minting.

Relevant boundary:
[`prepare_vote_change`](prog/src/instructions/vote_change.rs) calls the pure
`set_gauge_votes` transition before
[`withdraw_vote`](prog/src/instructions/withdraw_vote.rs) transfers tokens.

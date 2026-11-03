Actions are pure functions. Each accepts a state snapshot and returns a receipt
containing the next state and any reserve distributions. The caller performs
authorization, token transfers, burns, and state persistence together.

`donate_reserve` adds reserve units without changing supply or the schedule.

`redeem_at_floor` burns AVA at the floor rate, rounding gross reserve down to
whole units. It charges the redemption fee rounded up, then splits that fee
between the protocol (rounded up) and market owner (the remainder). Held reserve
decreases by the subject's payment plus both fees. Outstanding advances remain
unchanged, and the remaining position must still collateralize its advance.

Floor redemption consumes only the flat tail: it moves the ramp end left by the
amount burned while preserving ramp width, scalar, and floor. The stored ramp
end uses circulating-supply coordinates. A redemption that would move the ramp
start below zero is rejected. This operation does not contract the ramp.

Numeric conversion and range failures are returned as errors. Capacity and
payout calculations require a fixed-number driver with `IntegralRounding` so
that the final reserve amount is settled in whole token units.

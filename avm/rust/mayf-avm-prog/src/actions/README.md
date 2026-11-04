Actions are pure functions over normalized quantities. Each accepts a state
snapshot and returns next-state and reserve-distribution receipts. A simulator
can apply the receipt directly. A Solana host will authorize and settle it,
including token-unit rounding and persistence.

`donate_reserve` adds held reserve without changing calculator state.

`redeem_at_floor` delegates the quote and circulation update to
`mayf-avm-calculator`. It preserves progress and segmentation, checks position
collateral and held liquidity, applies fees, and enforces the minimum payout.
Fees round up on the numeric grid; the market owner receives the remainder
after the protocol share. The receipt conserves reserve on that grid.

See the [crate documentation](../../README.md) for the layer boundaries,
numeric contract, and a complete action example.

# AVM program

Pure program actions built on `mayf-avm-calculator`. Each action accepts a state
snapshot and returns a receipt with the next state and reserve distributions.
A simulator can apply these receipts directly; a host can settle them with
external accounts. The crate is `no_std` and performs no storage or token I/O.

## Layers

| Layer                      | Responsibility                                                                                                         |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Characteristic engine      | Bounded values and integrals of the immutable neutral function.                                                        |
| `mayf-avm-calculator`      | Segmentation, progress and circulation, quotes, contraction, and mathematical floor-raise verification.                |
| `mayf-avm-prog`            | Program actions, position collateral, reserve accounting, and fees, using generic normalized quantities.               |
| Future `mayf-avm-prog-sol` | Concrete numeric types, token normalization, authentication, Solana errors, CPI transport, transfers, and persistence. |

Borrowing, repayment, option exercise, ordinary trading, and semantic floor-raise
flavors belong in this program layer as it grows. The current actions are
`donate_reserve` and `redeem_at_floor`. For example, a future
`raise_floor_from_surplus` action will select a policy and establish reserve
funding before accepting the calculator's verified next state.

## State and arithmetic

`Market<N>` embeds the calculator's checked `MarketState<N>` and stores the
immutable ramp scalar, held and advanced reserves, and fee configuration.
Segmentation and supply counters have a single representation in calculator
state. `Position<N>` stores owned AVA and outstanding reserve advances.

All balances and quantities use `NonNeg<N>` or `Pos<N>`, in the same normalized
units as the calculator. Actions require positive amounts. The program selects
conservative bounds and rounds fees on the chosen fixed-number grid; it does
not round to whole token units or select a numeric implementation. The
calculator uses `SignedFixed` for intermediate arithmetic, while persisted
quantities remain nonnegative.

Fee rates use `MicroBasisPoints`. Fees round up with one fused
`amount * rate / 1_000_000` calculation, avoiding a pre-rounded rate. The protocol
share also rounds up, and the market owner receives the exact remainder. The
numeric type must represent the integer fee divisor `1_000_000`. A position's
advance capacity is its AVA multiplied by the floor, rounded down on the same
grid.

The caller binds a market to its characteristic parameters and ramp scalar.
For a Solana host, settlement must reconcile receipt amounts with actual token
rounding and retained remainders before persisting balances. Account validation,
engine authorization, and atomic settlement belong to that host.

## Actions

`donate_reserve` increases held reserves without changing calculator state,
outstanding advances, or positions.

`redeem_at_floor` obtains the gross quote and next calculator state from
`AvmCalculator::quote_redeem_at_floor`. It checks owned AVA, remaining collateral,
held reserve liquidity, and the minimum payout after fees. The gross reserve
amount uses the quote's lower bound. The receipt conserves reserve exactly on
the numeric grid: the reduction in held reserve equals the subject's payout
plus both fee shares.

Floor redemption changes circulation only. Progress, floor, ramp end, and ramp
width remain unchanged, so the current price is preserved. Available floor-tail
capacity comes from calculator state; the program does not translate geometry
or duplicate that calculation.

```rust
use mayf_avm_calculator::{Accounting, MarketState, Segmentation};
use mayf_avm_prog::{
    actions::{RedeemAtFloorArgs, redeem_at_floor},
    state::{FeeRates, Market, Position},
};
use mayf_fixed_drivers::I64F64;
use mayf_fixed_number::{AboveOne, NonNeg, Pos};
use mayf_micro_basis_points::MicroBasisPoints;

type N = I64F64;
let n = |value| NonNeg::<N>::from_u64(value).unwrap();
let quarter = MicroBasisPoints::from_percent(25).unwrap();
let calculator_state = MarketState::new(
    Segmentation::new(n(16), n(5), n(2)).unwrap(),
    Accounting::new(n(6), n(6)),
).unwrap();
let market = Market {
    calculator_state,
    ramp_scalar: AboveOne::new(n(2).get()).unwrap(),
    reserve_held: n(200),
    reserve_advanced: n(0),
    fee_rates: FeeRates {
        issue_fee: MicroBasisPoints::ZERO,
        redeem_fee: quarter,
        exercise_fee: MicroBasisPoints::ZERO,
        reserve_access_fee: MicroBasisPoints::ZERO,
    },
    protocol_fee_rate: quarter,
};
let receipt = redeem_at_floor(RedeemAtFloorArgs {
    amount_ava: Pos::from_i32(3).unwrap(),
    min_reserve_out: n(36),
    market,
    position: Position { ava_owned: n(6), reserve_advanced: n(0) },
}).unwrap();

assert_eq!(receipt.reserve_to_subject, n(36));
assert_eq!(receipt.fee_chain.fee_to_protocol, n(3));
assert_eq!(receipt.fee_chain.fee_to_market_owner, n(9));
assert_eq!(receipt.market_next.reserve_held, n(152));
assert_eq!(receipt.market_next.calculator_state.accounting().progress(), n(6));
assert_eq!(receipt.market_next.calculator_state.accounting().circulating_supply(), n(3));
assert_eq!(receipt.market_next.calculator_state.segmentation(), calculator_state.segmentation());
```

Run `cargo test -p mayf-avm-prog --locked` for the program action tests. Curve
integration and floor-raise arithmetic are tested in `mayf-avm-calculator`.

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

Borrowing, repayment, and semantic floor raises belong in this program layer
as it grows. For example, a future
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

`issue_ava` buys an exact amount of AVA. The upper bound from
`AvmCalculator::quote_issue` is credited to held reserves. The buyer pays
that amount plus the issue fee, subject to the inclusive `max_reserve_in`.
The receipt credits the AVA to the position and carries the calculator's
updated progress and circulation.

Pass the market's characteristic engine by reference. A simulator can use
`LocalCharacteristic` with the linear driver. An Anchor host can implement
`CharacteristicEngine::spans` with one synchronous CPI per successful issue.
The host checks the buyer's cap against the final token-unit debit and settles
the transfers, mint, and state updates atomically.

`issue_ava_at_floor` settles an option exercise. The holder receives the exact
AVA amount and deposits `floor * amount_ava` into held reserves, rounded up on
the fixed-point grid. The configured `exercise_fee` is charged separately and
counts toward `max_reserve_in`. Circulation and the floor tail grow by the
amount issued. Progress and ramp geometry stay fixed, which preserves the spot
price. The action requires no characteristic engine call. The host authorizes
and consumes the option when settling the receipt.

`redeem_ava` sells an exact amount of AVA. The calculator integrates the
pre-sale schedule across any main, ramp, and floor portions sold. Progress
decreases toward zero. Any AVA sold beyond that point pays the floor price.
If the sale ends inside the ramp, the ramp ends at the remaining progress.
If it ends on the floor, the ramp width becomes zero. The next buy starts on
main in both cases. Pricing takes at most one engine batch, and contraction
uses the geometry returned in the same quote.

`redeem_at_floor` sells directly from the available floor tail. It decreases
circulation and preserves progress, segmentation, and the current price.

Both redemptions check the seller's holdings and remaining collateral, use the
quote's lower bound for the gross reserve debit, and enforce `min_reserve_out`
on the payout after fees. Held reserves must cover the payout and both fee
shares. A Solana host enforces the minimum on the final token-unit payout.

`donate_reserve` increases held reserves without changing calculator state,
outstanding advances, or positions.

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

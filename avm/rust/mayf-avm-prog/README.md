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

Semantic floor raises belong in this program layer as it grows. For example, a future
`raise_floor_from_surplus` action will select a policy and establish reserve
funding before accepting the calculator's verified next state.

## State and arithmetic

`Market<N>` embeds the calculator's checked `MarketState<N>` and stores the
immutable ramp scalar, held and advanced reserves, and fee configuration.
Segmentation and supply counters have a single representation in calculator
state. `Position<N>` stores owned AVA and outstanding reserve advances.

Balances and quantities use `NonNeg<N>` or `Pos<N>`, in the same normalized
units as the calculator. Trades require positive amounts. Reconciliation accepts
zero targets and reports a signed net reserve transfer. The program selects
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

`reconcile_position` handles ordinary buying, selling, borrowing, and repayment
from target AVA holdings and outstanding advance. For a simple buy or sell,
keep the target advance equal to the current advance. It quotes the AVA
difference on the current schedule and nets it with the draw or repayment.
Collateral and liquidity checks apply to the combined result, allowing a
purchase to supply its own collateral or a sale to repay the advance securing it.

Buying uses the upper cost bound and the issue fee. Selling uses the lower
proceeds bound and the redemption fee. A sale is priced on the pre-sale
schedule. When it enters the ramp or floor, contraction attaches main at the
remaining progress. The next buy starts on main. Newly drawn reserve incurs the reserve access fee.
Repayment carries no fee. Trade and borrowing fees are split separately to
preserve each operation's protocol rounding.

The signed `reserve_to_subject` is the final transfer after fees. Positive means
the subject receives reserve, negative means they pay. `SlippageProtection`
supplies the initial wallet balance and its minimum acceptable final balance,
which limits either spending or the expected payout. The host supplies the
actual initial balance and enforces that minimum after token-unit rounding.

Reconciliation takes at most one synchronous characteristic batch. Borrowing
and repayment with constant AVA holdings require zero engine calls. A simulator
can use `LocalCharacteristic` with the linear driver. An Anchor host can supply
a `CharacteristicEngine::spans` implementation that performs CPI.

The two floor actions change circulation while preserving progress, ramp
geometry, and spot price. Both require zero characteristic engine calls.

- `issue_ava_at_floor` settles an option exercise. The holder receives the exact
  AVA amount and deposits `floor * amount_ava` into held reserves, rounded up.
  The exercise fee is charged separately and counts toward `max_reserve_in`.
  The host authorizes and consumes the option when settling the receipt.
- `redeem_at_floor` sells from the available floor tail. It checks holdings,
  remaining collateral, and held liquidity. The gross reserve debit uses the
  lower quote bound. The payout after the redemption fee must meet
  `min_reserve_out`.

`donate_reserve` increases held reserves. The host settles each action's
reserve transfers, fee distributions, token mint or burn, and next state atomically.

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

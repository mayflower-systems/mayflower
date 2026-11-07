use core::cell::Cell;

use mayf_avm_calculator::{
    Accounting, CharacteristicEngine, CharacteristicEngineError, CharacteristicEngineResult,
    Error as CalculatorError, LocalCharacteristic, MarketState, Segmentation, SpanRequest,
    SpanResponse,
};
use mayf_avm_prog::{
    actions::{
        ReconcilePositionArgs, ReconcilePositionError, SlippageProtection, reconcile_position,
    },
    fees::FeeChain,
    state::{FeeRates, Market, Position},
};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::I64Scaled;
use mayf_fixed_number::{AboveOne, FixedNumber, NonNeg, Pos};
use mayf_micro_basis_points::MicroBasisPoints;

type N = I64Scaled<10>;

fn n(value: u64) -> NonNeg<N> {
    NonNeg::from_u64(value).unwrap()
}

fn tenths(value: i64) -> NonNeg<N> {
    NonNeg::new(N::from_scaled_i64(value)).unwrap()
}

fn rate(percent: u32) -> MicroBasisPoints {
    MicroBasisPoints::from_percent(percent).unwrap()
}

#[derive(Default)]
struct Engine {
    calls: Cell<usize>,
    fail: bool,
}

impl CharacteristicEngine<N> for Engine {
    fn spans(
        &self,
        requests: &[SpanRequest<N>],
        responses: &mut [SpanResponse<N>],
    ) -> CharacteristicEngineResult<()> {
        self.calls.set(self.calls.get() + 1);
        if self.fail {
            return Err(CharacteristicEngineError::InvocationFailed);
        }
        LocalCharacteristic::new(LinearCharacteristic::new(Pos::from_i32(2).unwrap()))
            .spans(requests, responses)
    }
}

fn args(owned: u64, advanced: u64, held: u64) -> ReconcilePositionArgs<N> {
    ReconcilePositionArgs {
        target_position_ava_owned: n(owned),
        target_position_reserve_advanced: n(advanced),
        slippage_protection: SlippageProtection {
            subject_initial_reserve_balance: n(20),
            subject_minimum_final_reserve_balance: n(0),
        },
        market: Market {
            calculator_state: MarketState::new(
                Segmentation::launch(n(10)),
                Accounting::new(n(owned), n(owned)),
            )
            .unwrap(),
            ramp_scalar: AboveOne::new(n(2).get()).unwrap(),
            reserve_held: n(held),
            reserve_advanced: n(advanced),
            fee_rates: FeeRates {
                issue_fee: rate(0),
                redeem_fee: rate(0),
                exercise_fee: rate(0),
                reserve_access_fee: rate(0),
            },
            protocol_fee_rate: rate(25),
        },
        position: Position {
            ava_owned: n(owned),
            reserve_advanced: n(advanced),
        },
    }
}

#[test]
fn all_trade_and_advance_directions_conserve_reserve() {
    // On y = 10 + 2x, selling [1,2] returns 13 and buying [2,3] costs 15.
    for (owned, advanced, cash, held) in [
        (0, 0, 19, 0),
        (1, 0, 8, 11),
        (1, 5, 13, 6),
        (1, 10, 18, 1),
        (2, 0, -5, 24),
        (2, 5, 0, 19),
        (2, 10, 5, 14),
        (3, 0, -20, 39),
        (3, 5, -15, 34),
        (3, 10, -10, 29),
    ] {
        let mut args = args(2, 5, 19);
        args.target_position_ava_owned = n(owned);
        args.target_position_reserve_advanced = n(advanced);
        args.slippage_protection
            .subject_minimum_final_reserve_balance = n((20 + cash) as u64);
        let engine = Engine::default();
        let receipt = reconcile_position(args, &engine).unwrap();
        assert_eq!(
            receipt.reserve_to_subject,
            N::checked_from_i128(cash).unwrap()
        );
        assert_eq!(
            receipt.market_next,
            Market {
                calculator_state: MarketState::new(
                    Segmentation::launch(n(10)),
                    Accounting::new(n(owned), n(owned)),
                )
                .unwrap(),
                reserve_held: n(held),
                reserve_advanced: n(advanced),
                ..args.market
            }
        );
        assert_eq!(
            receipt.position_next,
            Position {
                ava_owned: n(owned),
                reserve_advanced: n(advanced)
            }
        );
        assert_eq!(receipt.borrow_fee_chain, FeeChain::default());
        assert_eq!(receipt.trade_fee_chain, FeeChain::default());
        assert_eq!(engine.calls.get(), usize::from(owned != 2));
    }
}

#[test]
fn trades_and_advances_fund_each_other_atomically() {
    // Opening needs only the 1-unit premium. The second buy also releases 7 cash.
    // The sale returns 13, repays 10, and succeeds with only 4 held initially.
    for (before, target, wallet, cash, held) in [
        ((0, 0, 0), (1, 10), 1, -1, 1),
        ((1, 0, 11), (2, 20), 0, 7, 4),
        ((2, 20, 4), (1, 10), 0, 3, 1),
    ] {
        let mut args = args(before.0, before.1, before.2);
        args.target_position_ava_owned = n(target.0);
        args.target_position_reserve_advanced = n(target.1);
        args.slippage_protection = SlippageProtection {
            subject_initial_reserve_balance: n(wallet),
            subject_minimum_final_reserve_balance: n((wallet as i128 + cash) as u64),
        };
        let engine = Engine::default();
        let receipt = reconcile_position(args, &engine).unwrap();
        assert_eq!(
            receipt.reserve_to_subject,
            N::checked_from_i128(cash).unwrap()
        );
        assert_eq!(receipt.market_next.reserve_held, n(held));
        assert_eq!(receipt.market_next.reserve_advanced, n(target.1));
        assert_eq!(receipt.position_next.ava_owned, n(target.0));
        assert_eq!(receipt.position_next.reserve_advanced, n(target.1));
        assert_eq!(engine.calls.get(), 1);
    }
}

#[test]
fn ramp_sale_repays_locked_collateral_and_prices_the_next_leveraged_buy() {
    let mut args = args(0, 0, 100);
    args.market.calculator_state = MarketState::new(
        Segmentation::new(n(3), n(4), n(2), n(3).get()).unwrap(),
        Accounting::new(tenths(52), tenths(92)),
    )
    .unwrap();
    args.market.reserve_advanced = n(20); // Includes another position's 2-unit advance.
    args.market.fee_rates.issue_fee = rate(2);
    args.market.fee_rates.redeem_fee = rate(25);
    args.market.fee_rates.reserve_access_fee = rate(2);
    args.position = Position {
        ava_owned: tenths(72),
        reserve_advanced: n(18),
    };
    args.target_position_ava_owned = n(5);
    args.target_position_reserve_advanced = n(10);
    args.slippage_protection = SlippageProtection {
        subject_initial_reserve_balance: n(0),
        subject_minimum_final_reserve_balance: tenths(97),
    };
    let engine = Engine::default();
    let sale = reconcile_position(args, &engine).unwrap();
    // The old ramp and main pay 23.64, rounded down to 23.6. Repay 8 and pay 5.9 fees.
    assert_eq!(sale.reserve_to_subject, tenths(97).get());
    assert_eq!(sale.market_next.reserve_held, tenths(844));
    assert_eq!(sale.market_next.reserve_advanced, n(12));
    assert_eq!(sale.borrow_fee_chain, FeeChain::default());
    assert_eq!(
        sale.trade_fee_chain,
        FeeChain {
            fee_to_protocol: tenths(15),
            fee_to_market_owner: tenths(44),
        }
    );
    assert_eq!(
        sale.market_next.calculator_state,
        MarketState::new(
            Segmentation::new(n(3), n(3), n(1), n(1).get()).unwrap(),
            Accounting::new(n(3), n(7)),
        )
        .unwrap()
    );
    assert_eq!(engine.calls.get(), 1);
    assert_eq!(
        reconcile_position(
            ReconcilePositionArgs {
                slippage_protection: SlippageProtection {
                    subject_minimum_final_reserve_balance: tenths(98),
                    ..args.slippage_protection
                },
                ..args
            },
            &Engine::default(),
        )
        .unwrap_err(),
        ReconcilePositionError::MinimumReserveBalanceNotMet
    );

    let engine = Engine::default();
    let buy_args = ReconcilePositionArgs {
        target_position_ava_owned: tenths(72),
        target_position_reserve_advanced: n(18),
        slippage_protection: SlippageProtection {
            subject_initial_reserve_balance: n(13),
            subject_minimum_final_reserve_balance: n(0),
        },
        market: sale.market_next,
        position: sale.position_next,
    };
    let buy = reconcile_position(buy_args, &engine).unwrap();
    // Contracted main costs 20.24, rounded up to 20.3. Draw 8 and pay fees 0.5 + 0.2.
    assert_eq!(buy.reserve_to_subject, N::checked_from_i128(-13).unwrap());
    assert_eq!(buy.market_next.reserve_held, tenths(967));
    assert_eq!(buy.market_next.reserve_advanced, n(20));
    assert_eq!(
        buy.trade_fee_chain,
        FeeChain {
            fee_to_protocol: tenths(2),
            fee_to_market_owner: tenths(3),
        }
    );
    assert_eq!(
        buy.borrow_fee_chain,
        FeeChain {
            fee_to_protocol: tenths(1),
            fee_to_market_owner: tenths(1),
        }
    );
    assert_eq!(engine.calls.get(), 1);
    assert_eq!(
        reconcile_position(
            ReconcilePositionArgs {
                slippage_protection: SlippageProtection {
                    subject_minimum_final_reserve_balance: tenths(1),
                    ..buy_args.slippage_protection
                },
                ..buy_args
            },
            &Engine::default(),
        )
        .unwrap_err(),
        ReconcilePositionError::MinimumReserveBalanceNotMet
    );
}

#[test]
fn sales_with_fixed_debt_collapse_the_ramp_and_exhaust_progress() {
    let mut args = args(0, 0, 100);
    args.market.calculator_state = MarketState::new(
        Segmentation::new(n(3), n(4), n(2), n(3).get()).unwrap(),
        Accounting::new(tenths(52), tenths(72)),
    )
    .unwrap();
    args.market.reserve_advanced = n(3);
    args.market.fee_rates.redeem_fee = rate(25);
    args.position = Position {
        ava_owned: tenths(72),
        reserve_advanced: n(3),
    };
    args.target_position_ava_owned = n(3);
    args.target_position_reserve_advanced = n(3);
    args.slippage_protection = SlippageProtection {
        subject_initial_reserve_balance: n(0),
        subject_minimum_final_reserve_balance: tenths(237),
    };
    let engine = Engine::default();
    let sale = reconcile_position(args, &engine).unwrap();
    // Floor pays 3, ramp pays 14, main pays 14.64. Settle 31.6 less the 7.9 fee.
    assert_eq!(sale.reserve_to_subject, tenths(237).get());
    assert_eq!(sale.market_next.reserve_held, tenths(684));
    assert_eq!(sale.market_next.reserve_advanced, n(3));
    assert_eq!(
        sale.position_next,
        Position {
            ava_owned: n(3),
            reserve_advanced: n(3)
        }
    );
    assert_eq!(sale.borrow_fee_chain, FeeChain::default());
    assert_eq!(
        sale.market_next.calculator_state,
        MarketState::new(
            Segmentation::new(n(3), n(1), n(0), n(1).get()).unwrap(),
            Accounting::new(n(1), n(3))
        )
        .unwrap()
    );
    assert_eq!(engine.calls.get(), 1);

    // Selling two more AVA consumes the final progress unit and one unit of excess supply.
    let engine = Engine {
        fail: true,
        ..Engine::default()
    };
    let sale = reconcile_position(
        ReconcilePositionArgs {
            target_position_ava_owned: n(1),
            slippage_protection: SlippageProtection {
                subject_initial_reserve_balance: n(0),
                subject_minimum_final_reserve_balance: tenths(45),
            },
            market: sale.market_next,
            position: sale.position_next,
            ..args
        },
        &engine,
    )
    .unwrap();
    assert_eq!(sale.reserve_to_subject, tenths(45).get());
    assert_eq!(
        sale.position_next,
        Position {
            ava_owned: n(1),
            reserve_advanced: n(3)
        }
    );
    assert_eq!(
        sale.market_next.calculator_state,
        MarketState::new(Segmentation::launch(n(3)), Accounting::new(n(0), n(1))).unwrap()
    );
    assert_eq!(engine.calls.get(), 0);
}

#[test]
fn netting_keeps_large_balances_and_separate_fees_representable() {
    let floor = 400_000_000_000_000_000;
    // The isolated purchase would overflow held reserve. Summing both fees or
    // inserting the target debt before removing the old debt can also overflow.
    for previous_advance in [0, floor] {
        let mut args = args(1, previous_advance, 600_000_000_000_000_000);
        args.market.calculator_state =
            MarketState::new(Segmentation::launch(n(floor)), Accounting::new(n(0), n(1))).unwrap();
        args.market.fee_rates.issue_fee = rate(100);
        args.market.fee_rates.reserve_access_fee = rate(100);
        args.market.protocol_fee_rate = rate(0);
        args.target_position_ava_owned = n(2);
        args.target_position_reserve_advanced = n(2 * floor);
        args.slippage_protection.subject_initial_reserve_balance = n(2 * (floor + 1));

        let receipt = reconcile_position(args, &Engine::default()).unwrap();
        // The purchase costs floor + 1. A 100% draw fee offsets the entire draw.
        assert_eq!(
            receipt.reserve_to_subject,
            N::checked_from_i128(-i128::from(2 * (floor + 1))).unwrap()
        );
        assert_eq!(
            receipt.market_next.reserve_held,
            n(200_000_000_000_000_001 + previous_advance)
        );
        assert_eq!(receipt.market_next.reserve_advanced, n(2 * floor));
        assert_eq!(receipt.trade_fee_chain.fee_to_market_owner, n(floor + 1));
        assert_eq!(
            receipt.borrow_fee_chain.fee_to_market_owner,
            n(2 * floor - previous_advance)
        );
    }
}

#[test]
fn reconciliation_rejects_unfunded_or_unhealthy_final_positions() {
    fn rejects(edit: impl FnOnce(&mut ReconcilePositionArgs<N>), error: ReconcilePositionError) {
        let mut args = args(2, 5, 19);
        edit(&mut args);
        assert_eq!(
            reconcile_position(args, &Engine::default()).unwrap_err(),
            error
        );
    }
    rejects(
        |args| {
            args.target_position_ava_owned = n(1);
            args.target_position_reserve_advanced = n(11);
        },
        ReconcilePositionError::PositionUnderCollateralized,
    );
    rejects(
        |args| {
            args.market.reserve_held = n(4);
            args.target_position_reserve_advanced = n(10);
        },
        ReconcilePositionError::ReserveBalanceTooLow,
    );
    rejects(
        |args| {
            args.target_position_ava_owned = n(3);
            args.slippage_protection.subject_initial_reserve_balance = n(14);
        },
        ReconcilePositionError::SubjectReserveBalanceTooLow,
    );
    rejects(
        |args| {
            args.target_position_ava_owned = n(3);
            args.slippage_protection
                .subject_minimum_final_reserve_balance = n(6);
        },
        ReconcilePositionError::MinimumReserveBalanceNotMet,
    );
    rejects(
        |args| args.market.reserve_advanced = n(4),
        ReconcilePositionError::PositionExceedsMarket,
    );
    let mut args = args(2, 5, 19);
    args.target_position_ava_owned = n(3);
    let engine = Engine {
        fail: true,
        ..Engine::default()
    };
    assert_eq!(
        reconcile_position(args, &engine).unwrap_err(),
        ReconcilePositionError::Calculator(CalculatorError::CharacteristicInvocationFailed)
    );
    assert_eq!(engine.calls.get(), 1);
}

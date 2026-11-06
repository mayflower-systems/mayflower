use core::fmt::Debug;

use mayf_avm_calculator::{
    Accounting, AvmCalculator, Error as CalculatorError, LocalCharacteristic, MarketState,
    Segmentation,
};
use mayf_avm_prog::{
    actions::{
        DonateReserveArgs, DonateReserveError, IssueAvaAtFloorArgs, IssueAvaAtFloorError,
        RedeemAtFloorArgs, RedeemAtFloorError, donate_reserve, issue_ava_at_floor, redeem_at_floor,
    },
    fees::{apply_fee, split_fee},
    state::{FeeRates, Market, Position, PositionError},
};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::{I64F64, I64Scaled};
use mayf_fixed_number::{AboveOne, DirectedRounding, FixedNumber, NonNeg, Pos, SignedFixed};
use mayf_micro_basis_points::MicroBasisPoints;

type Tenths = I64Scaled<10>;

fn n<N: FixedNumber>(value: u64) -> NonNeg<N> {
    NonNeg::from_u64(value).unwrap()
}

fn fraction<N: FixedNumber>(numerator: i128, denominator: u128) -> NonNeg<N> {
    NonNeg::new(
        N::checked_from_ratio_round(
            numerator,
            denominator.try_into().unwrap(),
            DirectedRounding::Floor,
        )
        .unwrap(),
    )
    .unwrap()
}

fn rate(percent: u32) -> MicroBasisPoints {
    MicroBasisPoints::from_percent(percent).unwrap()
}

fn linear_engine<N: FixedNumber>(slope: i32) -> LocalCharacteristic<LinearCharacteristic<N>> {
    LocalCharacteristic::new(LinearCharacteristic::new(Pos::from_i32(slope).unwrap()))
}

fn market<N: FixedNumber>() -> Market<N> {
    Market {
        calculator_state: MarketState::new(
            Segmentation::new(fraction(3, 2), n(10), fraction(5, 2)).unwrap(),
            Accounting::new(n(20), n(20)),
        )
        .unwrap(),
        ramp_scalar: AboveOne::new(n::<N>(2).get()).unwrap(),
        reserve_held: n(100),
        reserve_advanced: n(7),
        fee_rates: FeeRates {
            issue_fee: rate(1),
            redeem_fee: rate(25),
            exercise_fee: rate(2),
            reserve_access_fee: rate(3),
        },
        protocol_fee_rate: rate(25),
    }
}

fn redemption<N: FixedNumber>() -> RedeemAtFloorArgs<N> {
    RedeemAtFloorArgs {
        amount_ava: Pos::from_i32(5).unwrap(),
        min_reserve_out: n(5),
        market: market(),
        position: Position {
            ava_owned: n(10),
            reserve_advanced: n(7),
        },
    }
}

#[test]
fn donation_changes_only_held_reserve_and_checks_the_numeric_limit() {
    let original = market::<Tenths>();
    let max = NonNeg::new(Tenths::MAX).unwrap();
    let receipt = donate_reserve(DonateReserveArgs {
        amount_reserve: Pos::try_from(max.checked_sub(original.reserve_held).unwrap()).unwrap(),
        market: original,
    })
    .unwrap();
    assert_eq!(
        receipt.market_next,
        Market {
            reserve_held: max,
            ..original
        }
    );
    assert_eq!(
        donate_reserve(DonateReserveArgs {
            amount_reserve: Pos::new(Tenths::QUANTUM).unwrap(),
            market: receipt.market_next,
        })
        .err(),
        Some(DonateReserveError::ReserveBalanceOverflow)
    );
}

#[test]
fn floor_issuance_funds_the_floor_tail_and_preserves_spot_price() {
    let mut market = market::<Tenths>();
    let segmentation = market.calculator_state.segmentation();
    // Circulation just covers the nonfloor supply. The initial floor tail is empty.
    market.calculator_state =
        MarketState::new(segmentation, Accounting::new(n(20), fraction(125, 10))).unwrap();
    let args = IssueAvaAtFloorArgs {
        amount_ava: Pos::try_from(fraction(67, 10)).unwrap(),
        max_reserve_in: fraction(104, 10),
        market,
        position: Position {
            ava_owned: n(10),
            reserve_advanced: n(7),
        },
    };
    let receipt = issue_ava_at_floor(args).unwrap();

    // 6.7 AVA at a floor of 1.5 needs 10.05 reserve, rounded up to 10.1.
    // The 2% exercise fee rounds to 0.3, split 0.1 to the protocol and 0.2 to the owner.
    assert_eq!(receipt.reserve_from_subject, fraction(104, 10));
    assert_eq!(receipt.fee_chain.fee_to_protocol, fraction(1, 10));
    assert_eq!(receipt.fee_chain.fee_to_market_owner, fraction(2, 10));
    assert_eq!(
        receipt.position_next,
        Position {
            ava_owned: fraction(167, 10),
            ..args.position
        }
    );
    assert_eq!(
        receipt.market_next,
        Market {
            calculator_state: MarketState::new(
                segmentation,
                Accounting::new(n(20), fraction(192, 10))
            )
            .unwrap(),
            reserve_held: fraction(1101, 10),
            ..market
        }
    );
    let next = receipt.market_next.calculator_state;
    assert_eq!(next.floor_tail().unwrap(), NonNeg::from(args.amount_ava));
    let calculator = AvmCalculator::new(market.ramp_scalar);
    let engine = linear_engine(2);
    assert_eq!(
        calculator.price(&next, &engine).unwrap(),
        calculator.price(&market.calculator_state, &engine).unwrap()
    );
    assert_eq!(
        issue_ava_at_floor(IssueAvaAtFloorArgs {
            max_reserve_in: fraction(103, 10),
            ..args
        })
        .unwrap_err(),
        IssueAvaAtFloorError::MaximumReserveInExceeded
    );
}

fn check_floor_redemption<N: SignedFixed + Debug>(
    payout: NonNeg<N>,
    protocol_fee: NonNeg<N>,
    owner_fee: NonNeg<N>,
) {
    let original = market::<N>();
    let engine = linear_engine(8);
    let calculator = AvmCalculator::new(original.ramp_scalar);
    let price = calculator
        .price(&original.calculator_state, &engine)
        .unwrap();
    let receipt = redeem_at_floor(redemption::<N>()).unwrap();

    assert_eq!(receipt.reserve_to_subject, payout);
    assert_eq!(receipt.fee_chain.fee_to_protocol, protocol_fee);
    assert_eq!(receipt.fee_chain.fee_to_market_owner, owner_fee);
    assert_eq!(
        receipt
            .market_next
            .reserve_held
            .checked_add(payout)
            .unwrap()
            .checked_add(protocol_fee)
            .unwrap()
            .checked_add(owner_fee)
            .unwrap(),
        original.reserve_held
    );
    assert_eq!(
        receipt.market_next,
        Market {
            calculator_state: MarketState::new(
                original.calculator_state.segmentation(),
                Accounting::new(n(20), n(15)),
            )
            .unwrap(),
            reserve_held: fraction(185, 2),
            ..original
        }
    );
    assert_eq!(
        receipt.position_next,
        Position {
            ava_owned: n(5),
            reserve_advanced: n(7),
        }
    );
    assert_eq!(
        calculator
            .price(&receipt.market_next.calculator_state, &engine)
            .unwrap(),
        price
    );
    assert!(receipt.position_next.is_healthy(fraction(3, 2)).unwrap());
}

#[test]
fn floor_redemption_conserves_reserve_and_preserves_price_and_progress() {
    // Gross reserve is 7.5. Fees round to the selected number's grid, not integers.
    check_floor_redemption::<I64F64>(fraction(45, 8), fraction(15, 32), fraction(45, 32));
    check_floor_redemption::<Tenths>(fraction(56, 10), fraction(5, 10), fraction(14, 10));

    let mut args = redemption::<Tenths>();
    args.amount_ava = Pos::try_from(fraction(3, 10)).unwrap();
    args.min_reserve_out = n(0);
    let receipt = redeem_at_floor(args).unwrap();
    // The exact gross 0.45 is enclosed by [0.4, 0.5]; settlement uses 0.4.
    assert_eq!(receipt.market_next.reserve_held, fraction(996, 10));
    assert_eq!(receipt.reserve_to_subject, fraction(3, 10));
    assert_eq!(receipt.fee_chain.fee_to_protocol, fraction(1, 10));
    assert_eq!(receipt.fee_chain.fee_to_market_owner, n(0));
}

#[test]
fn floor_tail_uses_circulation_separately_from_progress() {
    for (width, tail) in [(fraction(5, 2), fraction(9, 2)), (n(0), n(7))] {
        let mut args = redemption::<I64F64>();
        args.market.calculator_state = MarketState::new(
            Segmentation::new(fraction(3, 2), n(10), width).unwrap(),
            Accounting::new(n(20), n(17)),
        )
        .unwrap();
        args.position.ava_owned = n(17);
        args.position.reserve_advanced = n(0);
        args.min_reserve_out = n(0);
        args.amount_ava = Pos::try_from(tail).unwrap();
        let receipt = redeem_at_floor(args).unwrap();
        assert_eq!(
            receipt.market_next.calculator_state.floor_tail().unwrap(),
            n(0)
        );
        assert_eq!(
            receipt.market_next.calculator_state.segmentation(),
            args.market.calculator_state.segmentation()
        );

        args.amount_ava = Pos::try_from(
            tail.checked_add(NonNeg::new(I64F64::QUANTUM).unwrap())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            redeem_at_floor(args).err(),
            Some(RedeemAtFloorError::Calculator(
                CalculatorError::InsufficientFloorTail
            ))
        );
    }
}

#[test]
fn fees_and_collateral_match_raw_integer_oracles_on_a_coarse_grid() {
    for raw_amount in [0, 1, 3, 17, 123, i64::MAX] {
        let amount = NonNeg::new(Tenths::from_scaled_i64(raw_amount)).unwrap();
        for raw_rate in [0, 1, 333_333, 1_000_000] {
            let rate = MicroBasisPoints::new(raw_rate).unwrap();
            let expected = (raw_amount as u128 * u128::from(raw_rate)).div_ceil(1_000_000);
            let fee = apply_fee(amount, rate).unwrap();
            assert_eq!(fee.get().to_scaled_i64() as u128, expected);
            let split = split_fee(amount, rate).unwrap();
            assert_eq!(split.fee_to_protocol, fee);
            assert_eq!(
                split
                    .fee_to_protocol
                    .checked_add(split.fee_to_market_owner)
                    .unwrap(),
                amount
            );
        }

        for raw_floor in [0, 1, 15] {
            let floor = NonNeg::new(Tenths::from_scaled_i64(raw_floor)).unwrap();
            let expected = i128::from(raw_amount) * i128::from(raw_floor) / 10;
            let mut position = Position {
                ava_owned: amount,
                reserve_advanced: n(0),
            };
            match i64::try_from(expected) {
                Ok(raw_capacity) => {
                    let capacity = NonNeg::new(Tenths::from_scaled_i64(raw_capacity)).unwrap();
                    assert_eq!(position.reserve_advance_capacity(floor), Ok(capacity));
                    position.reserve_advanced = capacity;
                    assert_eq!(position.is_healthy(floor), Ok(true));
                    position.reserve_advanced = capacity
                        .checked_add(NonNeg::new(Tenths::QUANTUM).unwrap())
                        .unwrap();
                    assert_eq!(position.is_healthy(floor), Ok(false));
                }
                Err(_) => assert_eq!(
                    position.reserve_advance_capacity(floor),
                    Err(PositionError::ArithmeticFailure)
                ),
            }
        }
    }
}

#[test]
fn redemption_rejects_insufficient_holdings_collateral_liquidity_and_slippage() {
    fn rejects(edit: impl FnOnce(&mut RedeemAtFloorArgs<I64F64>), error: RedeemAtFloorError) {
        let mut args = redemption();
        edit(&mut args);
        assert_eq!(redeem_at_floor(args).err(), Some(error));
    }

    rejects(
        |args| args.amount_ava = Pos::from_i32(11).unwrap(),
        RedeemAtFloorError::AvaBalanceTooLow,
    );
    rejects(
        |args| args.position.reserve_advanced = n(8),
        RedeemAtFloorError::PositionUnderCollateralized,
    );
    rejects(
        |args| args.market.reserve_held = n(7),
        RedeemAtFloorError::ReserveBalanceTooLow,
    );
    rejects(
        |args| args.min_reserve_out = n(6),
        RedeemAtFloorError::MinimumReserveOutNotMet,
    );
}

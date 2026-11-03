use mayf_avm_prog::{
    actions::{
        DonateReserveArgs, DonateReserveError, RedeemAtFloorArgs, RedeemAtFloorError,
        donate_reserve, redeem_at_floor,
    },
    state::{FeeRates, Market, Position, PositionError, SegmentationGeometry},
};
use mayf_fixed_drivers::U64F64;
use mayf_fixed_number::{AboveOne, NonNeg};
use mayf_micro_basis_points::MicroBasisPoints;

type RedeemArgs = RedeemAtFloorArgs<U64F64>;

fn rate(percent: u32) -> MicroBasisPoints {
    MicroBasisPoints::from_percent(percent).unwrap()
}

fn market() -> Market<U64F64> {
    Market {
        segmentation_geometry: SegmentationGeometry {
            floor: NonNeg::from_unsigned(U64F64::from_scaled_u128(3 << 63)), // 1.5
            ramp_width: NonNeg::from_unsigned(U64F64::from_scaled_u128(5 << 63)), // 2.5
            ramp_scalar: AboveOne::new(U64F64::from_u64(2)).unwrap(),
            ramp_end: 10,
        },
        supply_ava: 20,
        reserve_held: 100,
        reserve_advanced: 7,
        fee_rates: FeeRates {
            issue_fee: rate(1),
            redeem_fee: rate(25),
            exercise_fee: rate(2),
            reserve_access_fee: rate(3),
        },
        protocol_fee_rate: rate(25),
    }
}

fn redemption() -> RedeemArgs {
    RedeemAtFloorArgs {
        amount_ava: 5,
        min_reserve_out: 5,
        market: market(),
        position: Position {
            ava_owned: 10,
            reserve_advanced: 7,
        },
    }
}

#[test]
fn donation_checks_reserve_overflow() {
    let receipt = donate_reserve(DonateReserveArgs {
        amount_reserve: u64::MAX - 100,
        market: market(),
    })
    .unwrap();
    let next = receipt.market_next;
    assert_eq!(next.reserve_held, u64::MAX);
    assert_eq!(next.supply_ava, 20);
    assert_eq!(next.segmentation_geometry.ramp_end, 10);

    assert_eq!(
        donate_reserve(DonateReserveArgs {
            amount_reserve: 1,
            market: next,
        })
        .err(),
        Some(DonateReserveError::ReserveBalanceOverflow)
    );
}

#[test]
fn redemption_conserves_reserve_and_preserves_the_remaining_collateral() {
    let receipt = redeem_at_floor(redemption()).unwrap();
    // Gross floor value is floor(5 * 1.5) = 7, fee is ceil(7 * 25%) = 2.
    assert_eq!(receipt.reserve_to_subject, 5);
    assert_eq!(receipt.fee_chain.fee_to_protocol, 1);
    assert_eq!(receipt.fee_chain.fee_to_market_owner, 1);
    assert_eq!(
        receipt.market_next.reserve_held
            + receipt.reserve_to_subject
            + receipt.fee_chain.fee_to_protocol
            + receipt.fee_chain.fee_to_market_owner,
        100
    );
    assert_eq!(receipt.market_next.supply_ava, 15);
    assert_eq!(receipt.position_next.ava_owned, 5);
    assert_eq!(receipt.position_next.reserve_advanced, 7);
    assert_eq!(receipt.market_next.reserve_advanced, 7);
    let geometry = receipt.market_next.segmentation_geometry;
    assert_eq!(geometry.ramp_end, 5);
    assert_eq!(geometry.floor, market().segmentation_geometry.floor);
    assert_eq!(
        geometry.ramp_width,
        market().segmentation_geometry.ramp_width
    );
    // The current supply's distance from the ramp is unchanged by translation.
    assert_eq!(receipt.market_next.supply_ava - geometry.ramp_end, 20 - 10);
}

#[test]
fn floor_tail_limits_are_exact_for_fractional_and_zero_ramp_widths() {
    for (width, allowed) in [(5 << 63, 7), (0, 10)] {
        for amount in [0, allowed, allowed + 1] {
            let mut args = redemption();
            args.market.segmentation_geometry.ramp_width =
                NonNeg::from_unsigned(U64F64::from_scaled_u128(width));
            args.position.ava_owned = 20;
            args.position.reserve_advanced = 0;
            args.amount_ava = amount;
            args.min_reserve_out = 0;
            if amount <= allowed {
                let receipt = redeem_at_floor(args).unwrap();
                assert_eq!(
                    receipt.market_next.segmentation_geometry.ramp_end,
                    10 - amount
                );
                assert_eq!(
                    receipt
                        .market_next
                        .segmentation_geometry
                        .ramp_width
                        .get()
                        .to_scaled_u128(),
                    width
                );
            } else {
                assert_eq!(
                    redeem_at_floor(args).err(),
                    Some(RedeemAtFloorError::FloorLiquidityTooLow)
                );
            }
        }
    }
}

#[test]
fn capacity_and_settlement_match_exact_integer_oracles_at_token_limits() {
    let unit = 1u128 << 64;
    let floors = [0, 1, unit / 2, 3 * unit / 2, unit + 1, u128::MAX];
    for raw in floors {
        for amount in [0, 1, 3, u64::MAX] {
            // Decompose the raw rate to avoid overflowing the independent oracle.
            let exact_whole =
                u128::from(amount) * (raw >> 64) + ((u128::from(amount) * (raw % unit)) >> 64);
            let floor = NonNeg::from_unsigned(U64F64::from_scaled_u128(raw));
            let position = Position {
                ava_owned: amount,
                reserve_advanced: 1,
            };
            let capacity = position.reserve_advance_capacity(floor);

            let mut args = redemption();
            args.amount_ava = amount;
            args.min_reserve_out = 0;
            args.market.supply_ava = amount;
            args.market.reserve_held = u64::MAX;
            args.market.reserve_advanced = 0;
            args.market.segmentation_geometry.floor = floor;
            args.market.segmentation_geometry.ramp_end = amount;
            args.market.segmentation_geometry.ramp_width = NonNeg::zero();
            args.position = Position {
                ava_owned: amount,
                reserve_advanced: 0,
            };
            match u64::try_from(exact_whole) {
                Ok(gross) => {
                    assert_eq!(capacity, Ok(gross));
                    assert_eq!(position.is_healthy(floor), Ok(gross >= 1));
                    let receipt = redeem_at_floor(args).unwrap();
                    let fee = u128::from(gross).div_ceil(4) as u64;
                    let protocol_fee = u128::from(fee).div_ceil(4) as u64;
                    assert_eq!(receipt.reserve_to_subject, gross - fee);
                    assert_eq!(receipt.fee_chain.fee_to_protocol, protocol_fee);
                    assert_eq!(receipt.fee_chain.fee_to_market_owner, fee - protocol_fee);
                    assert_eq!(receipt.market_next.reserve_held, u64::MAX - gross);
                }
                Err(_) => {
                    assert_eq!(capacity, Err(PositionError::ArithmeticFailure));
                    assert_eq!(
                        position.is_healthy(floor),
                        Err(PositionError::ArithmeticFailure)
                    );
                    assert_eq!(
                        redeem_at_floor(args).err(),
                        Some(RedeemAtFloorError::ArithmeticFailure)
                    );
                }
            }
        }
    }
}

#[test]
fn redemption_rejects_unfunded_unsecured_and_out_of_bounds_requests() {
    fn rejects(edit: impl FnOnce(&mut RedeemArgs), error: RedeemAtFloorError) {
        let mut args = redemption();
        edit(&mut args);
        assert_eq!(redeem_at_floor(args).err(), Some(error));
    }

    rejects(
        |args| args.amount_ava = 11,
        RedeemAtFloorError::AvaBalanceTooLow,
    );
    rejects(
        |args| args.market.supply_ava = 4,
        RedeemAtFloorError::SupplyTooLow,
    );
    rejects(
        |args| args.market.segmentation_geometry.ramp_end = 21,
        RedeemAtFloorError::InvalidSegmentation,
    );
    rejects(
        |args| args.market.segmentation_geometry.ramp_end = 4,
        RedeemAtFloorError::FloorLiquidityTooLow,
    );
    rejects(
        |args| args.position.reserve_advanced = 8,
        RedeemAtFloorError::PositionUnderCollateralized,
    );
    rejects(
        |args| args.market.reserve_held = 6,
        RedeemAtFloorError::ReserveBalanceTooLow,
    );
    rejects(
        |args| args.min_reserve_out = 6,
        RedeemAtFloorError::MinimumReserveOutNotMet,
    );
}

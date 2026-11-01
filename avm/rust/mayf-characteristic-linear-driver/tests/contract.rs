use mayf_characteristic::{AffineAnchor, AffineTransform, Characteristic};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_drivers::{I64F64, I64Scaled};
use mayf_fixed_number::{Bounds, FixedNumber, NonNeg, Pos};
use mayf_monotone::{MonotoneCalculator, MonotoneCalculatorError};

type Fx = I64F64;
fn nn(value: i128) -> NonNeg<Fx> {
    NonNeg::new(Fx::checked_from_i128(value).unwrap()).unwrap()
}

#[test]
fn domain_and_overflow_failures_do_not_produce_quotes() {
    let curve = LinearCharacteristic::new(Pos::<Fx>::one());
    let view = |translation, anchor_x, anchor_y| {
        curve.calculator(AffineTransform::new(
            Pos::one(),
            translation,
            nn(1),
            AffineAnchor::exact(anchor_x, anchor_y),
        ))
    };
    let valid = view(nn(0).get(), nn(2), nn(4));
    assert_eq!(
        valid.y(nn(1)),
        Err(MonotoneCalculatorError::InputOutOfDomain)
    );
    assert_eq!(
        valid.integrate(nn(1), nn(1)),
        Err(MonotoneCalculatorError::InputOutOfDomain)
    );

    let invalid_anchor = view(Fx::checked_from_i128(-2).unwrap(), nn(1), nn(4));
    assert_eq!(
        invalid_anchor.y(nn(1)),
        Err(MonotoneCalculatorError::InputOutOfDomain)
    );
    assert_eq!(
        invalid_anchor.y(nn(2)),
        Err(MonotoneCalculatorError::InputOutOfDomain)
    );
    assert_eq!(
        invalid_anchor.integrate(nn(1), nn(1)),
        Err(MonotoneCalculatorError::InputOutOfDomain)
    );
    // The trait explicitly requires exact zero even outside the view's domain.
    assert_eq!(
        invalid_anchor.integrate(nn(0), nn(0)),
        Ok(Bounds::exact(nn(0)))
    );

    let max = NonNeg::new(Fx::MAX).unwrap();
    for calculator in [
        view(Fx::MAX, nn(0), nn(0)), // transformed coordinate overflows
        view(Fx::ZERO, nn(0), max),  // output overflows
        LinearCharacteristic::try_new(Fx::MAX)
            .unwrap()
            .calculator(AffineTransform::new(
                Pos::new(nn(2).get()).unwrap(),
                Fx::ZERO,
                nn(1),
                AffineAnchor::exact(nn(0), nn(0)),
            )), // derived slope overflows
    ] {
        assert_eq!(
            calculator.y(nn(1)),
            Err(MonotoneCalculatorError::ArithmeticFailure)
        );
        assert_eq!(
            calculator.integrate(nn(0), nn(2)),
            Err(MonotoneCalculatorError::ArithmeticFailure)
        );
    }
    let identity = curve.calculator(AffineTransform::identity());
    assert_eq!(
        identity.integrate(max, nn(1)),
        Err(MonotoneCalculatorError::ArithmeticFailure)
    );
}

#[test]
fn zero_vertical_scale_integrates_the_anchor_enclosure_without_slope_overflow() {
    let anchor_y = Bounds::new(nn(4), nn(5)).unwrap();
    let calc = LinearCharacteristic::try_new(Fx::MAX)
        .unwrap()
        .calculator(AffineTransform::new(
            Pos::new(nn(2).get()).unwrap(),
            Fx::ZERO,
            nn(0),
            AffineAnchor::new(nn(1), anchor_y),
        ));
    assert_eq!(calc.y(nn(3)).unwrap(), anchor_y);
    assert_eq!(
        calc.integrate(nn(1), nn(2)).unwrap(),
        Bounds::new(nn(8), nn(10)).unwrap()
    );
}

#[test]
fn solvers_find_the_greatest_safe_endpoint_on_every_cell_of_a_small_grid() {
    type Grid = I64Scaled<10>;
    let raw = |value| NonNeg::new(Grid::from_scaled_i64(value)).unwrap();
    for transformed in [false, true] {
        let anchor_raw = if transformed { 4 } else { 0 };
        let view = if transformed {
            AffineTransform::new(
                Pos::new(raw(13).get()).unwrap(),
                Grid::from_scaled_i64(-3),
                raw(7),
                AffineAnchor::new(raw(anchor_raw), Bounds::new(raw(2), raw(3)).unwrap()),
            )
        } else {
            AffineTransform::identity()
        };
        for slope_raw in 1..=30 {
            let calc =
                LinearCharacteristic::new(Pos::new(raw(slope_raw).get()).unwrap()).calculator(view);
            // Exhaustive endpoints expose quantization plateaus and test actual
            // maximality; checking only the returned quote's safety misses gaps.
            for hi_raw in 1..=30 {
                for target_raw in 1..=50 {
                    let target = raw(target_raw);
                    let lo = raw(anchor_raw);
                    let hi = raw(anchor_raw + hi_raw);
                    let buy = calc
                        .max_hi_for_area_at_or_below(lo, Some(hi), target)
                        .unwrap();
                    let expected_hi = (1..=hi_raw)
                        .rev()
                        .find(|&end| calc.integrate(lo, raw(end)).unwrap().upper() <= target);
                    assert_eq!(
                        buy.map(|s| s.endpoint),
                        expected_hi.map(|end| raw(anchor_raw + end)),
                        "buy: slope={slope_raw} hi={hi_raw} target={target_raw} transformed={transformed}"
                    );
                    if let Some(solution) = buy {
                        assert_eq!(
                            solution.area,
                            calc.integrate(lo, solution.endpoint.checked_sub(lo).unwrap())
                                .unwrap()
                                .upper()
                        );
                    }
                    let sell = calc.max_lo_for_area_at_or_above(lo, hi, target).unwrap();
                    let expected_lo = (0..hi_raw).rev().find(|&start| {
                        calc.integrate(raw(anchor_raw + start), raw(hi_raw - start))
                            .unwrap()
                            .lower()
                            >= target
                    });
                    assert_eq!(
                        sell.map(|s| s.endpoint),
                        expected_lo.map(|start| raw(anchor_raw + start)),
                        "sell: slope={slope_raw} hi={hi_raw} target={target_raw} transformed={transformed}"
                    );
                    if let Some(solution) = sell {
                        assert_eq!(
                            solution.area,
                            calc.integrate(
                                solution.endpoint,
                                hi.checked_sub(solution.endpoint).unwrap()
                            )
                            .unwrap()
                            .lower()
                        );
                    }
                }
            }
        }
    }
    let calc =
        LinearCharacteristic::new(Pos::<Grid>::one()).calculator(AffineTransform::identity());
    let budget = raw(5);
    let unlimited = calc
        .max_hi_for_area_at_or_below(raw(0), None, budget)
        .unwrap()
        .unwrap();
    let limited = calc
        .max_hi_for_area_at_or_below(raw(0), Some(raw(30)), budget)
        .unwrap()
        .unwrap();
    assert_eq!(unlimited, limited);
    assert_eq!(
        calc.max_hi_for_area_at_or_below(raw(7), None, raw(0))
            .unwrap()
            .unwrap()
            .endpoint,
        raw(7)
    );
    assert_eq!(
        calc.max_lo_for_area_at_or_above(raw(0), raw(7), raw(0))
            .unwrap()
            .unwrap()
            .endpoint,
        raw(7)
    );
}

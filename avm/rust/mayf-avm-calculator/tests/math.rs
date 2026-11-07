//! Raw-integer oracles independent of the calculator's interval splitting.
use core::fmt::Debug;

use dashu::{base::DivRem, integer::UBig};
use mayf_avm_calculator::{
    Accounting, AvmCalculator, FloorRaisePolicy, FloorRaiseProposal, LocalCharacteristic,
    MarketState, ScheduleCalculator, Segmentation,
};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_drivers::{I64F64, I64Scaled};
use mayf_fixed_number::{AboveOne, Bounds, NonNeg, Pos, SignedFixed};

#[derive(Clone, Copy, Debug)]
struct Curve {
    floor: i128,
    slope: i128,
    scalar: i128,
    start: i128,
    end: i128,
}

fn big(value: i128) -> UBig {
    UBig::from(u128::try_from(value).unwrap())
}

impl Curve {
    // Native raw price units, with denominator S^2.
    fn price(self, x: i128, s: &UBig) -> UBig {
        let ramp = (x - self.start).max(0).min(self.end - self.start);
        let main = (x - self.end).max(0);
        big(self.floor) * s * s
            + big(self.slope) * big(self.scalar) * big(ramp)
            + big(self.slope) * s * big(main)
    }

    // Native raw area units, with denominator 2*S^3. Positive-part squares
    // express the whole curve without duplicating the implementation's cursor.
    fn primitive(self, x: i128, s: &UBig) -> UBig {
        let start = big((x - self.start).max(0));
        let end = big((x - self.end).max(0));
        let start_square = &start * &start;
        let end_square = &end * &end;
        2u32 * big(self.floor) * big(x) * s * s
            + big(self.slope) * big(self.scalar) * (start_square - &end_square)
            + big(self.slope) * s * end_square
    }
}

fn outward(numerator: UBig, denominator: &UBig) -> (i128, i128) {
    let (floor, remainder) = numerator.div_rem(denominator);
    let floor = i128::try_from(floor).unwrap();
    (floor, floor + i128::from(remainder != UBig::ZERO))
}

fn check_bounds(actual: (i128, i128), ideal: (i128, i128), slack: i128) {
    assert!(
        actual.0 <= ideal.0 && ideal.1 <= actual.1,
        "{actual:?} excludes {ideal:?}"
    );
    assert!(
        ideal.0 - actual.0 <= slack && actual.1 - ideal.1 <= slack,
        "{actual:?} is too loose around {ideal:?}; allowance {slack} quanta"
    );
}

fn check_grid<N: SignedFixed + Debug>(scale: i128, from_raw: fn(i128) -> N, to_raw: fn(N) -> i128) {
    let nn = |raw| NonNeg::new(from_raw(raw)).unwrap();
    let pos = |raw| Pos::new(from_raw(raw)).unwrap();
    let edges =
        |bounds: Bounds<NonNeg<N>>| (to_raw(bounds.lower().get()), to_raw(bounds.upper().get()));
    let s = big(scale);
    let price_denominator = &s * &s;
    let area_denominator = 2u32 * &s * &s * &s;
    let mut seed = 0x7365_676d_656e_7473u128;
    let mut sample = |limit: i128| {
        seed = seed
            .wrapping_mul(0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645)
            .wrapping_add(1);
        (seed % limit as u128) as i128
    };
    for case in 0..128 {
        // Keep the independently stored offset representable while varying
        // slope, scalar, and boundaries across the number library's full grid.
        let (slope, scalar, coordinate_step) = match case % 3 {
            0 => (1 + sample(4 * scale), 2 * scale, scale),
            1 => (
                (1 + sample(4)) * scale,
                scale + 1 + sample(2 * scale),
                scale,
            ),
            _ => ((1 + sample(4)) * scale, 2 * scale, 1),
        };
        let start = if case % 16 == 0 {
            0
        } else {
            sample(4 * scale / coordinate_step) * coordinate_step
        };
        let curve = Curve {
            floor: 1 + sample(8 * scale),
            slope,
            scalar,
            start,
            end: start
                + if case % 8 == 0 {
                    0
                } else {
                    sample(4 * scale / coordinate_step) * coordinate_step
                },
        };
        let offset = curve.floor
            + match case % 3 {
                0 => slope * (curve.end / scale - 2 * curve.start / scale),
                1 => {
                    (slope / scale)
                        * ((scalar - scale) * (curve.end / scale) - scalar * (curve.start / scale))
                }
                _ => (slope / scale) * (curve.end - 2 * curve.start),
            };
        let segmentation = Segmentation::new(
            nn(curve.floor),
            nn(curve.end),
            nn(curve.end - curve.start),
            from_raw(offset),
        )
        .unwrap();
        let engine = LocalCharacteristic::new(LinearCharacteristic::new(pos(curve.slope)));
        let scalar = AboveOne::new(from_raw(curve.scalar)).unwrap();
        let calculator = ScheduleCalculator::new(segmentation, scalar);
        for x in [
            0,
            (curve.start - 1).max(0),
            curve.start,
            curve.start + 1,
            (curve.end - 1).max(0),
            curve.end,
            curve.end + 1,
            curve.end + 3 * scale,
        ] {
            // With k < 3, neutral endpoint bounds and the ramp-scale product
            // must remain within eight quanta of the exact price.
            check_bounds(
                edges(calculator.y(nn(x), &engine).unwrap_or_else(|error| {
                    panic!("price failed: {error:?}; scale={scale}, x={x}, curve={curve:?}")
                })),
                outward(curve.price(x, &s), &price_denominator),
                8,
            );
        }
        for (lo, width) in [
            (0, curve.end + 3 * scale),
            ((curve.start - 1).max(0), 2),
            ((curve.end - 1).max(0), 2),
            (curve.start, curve.end - curve.start),
            (curve.end, 3 * scale),
            (curve.end + 1, 1),
        ] {
            let ideal = outward(
                curve.primitive(lo + width, &s) - curve.primitive(lo, &s),
                &area_denominator,
            );
            // Check that scaling and rectangle adjustments retain a tight
            // enclosure across these normalized ramp and main widths.
            check_bounds(
                edges(calculator.integrate(nn(lo), nn(width), &engine).unwrap_or_else(|error| {
                    panic!("area failed: {error:?}; scale={scale}, lo={lo}, width={width}, curve={curve:?}")
                })),
                ideal,
                32,
            );
        }

        for remaining in [
            0,
            curve.start,
            curve.start + ((curve.end - curve.start) / (2 * scale)) * scale,
            curve.end,
            curve.end + scale,
        ] {
            let progress = curve.end + 2 * scale;
            let state = MarketState::new(segmentation, Accounting::new(nn(progress), nn(progress)))
                .unwrap();
            let contracted_state = AvmCalculator::new(scalar)
                .quote_redeem(&state, pos(progress - remaining), &engine)
                .unwrap()
                .next_state
                .segmentation();
            let expected = Curve {
                start: curve.start.min(remaining),
                end: curve.end.min(remaining),
                ..curve
            };
            let contracted = ScheduleCalculator::new(contracted_state, scalar);
            // Both geometries retain the same exact prefix. Their independently
            // assembled enclosures need not have identical endpoints.
            let prefix = outward(curve.primitive(remaining, &s), &area_denominator);
            for schedule in [&calculator, &contracted] {
                check_bounds(
                    edges(schedule.integrate(nn(0), nn(remaining), &engine).unwrap()),
                    prefix,
                    32,
                );
            }
            for x in [remaining, curve.end + scale] {
                check_bounds(
                    edges(contracted.y(nn(x), &engine).unwrap()),
                    outward(expected.price(x, &s), &price_denominator),
                    8,
                );
            }
        }

        // Move both joins by one normalized unit and raise the floor by the
        // linear slope. This preserves main exactly; extra circulation funds
        // the increased backing. Compare the direct residual against two
        // independently integrated whole curves, including rounded endpoints.
        let shifted = Curve {
            floor: curve.floor + curve.slope,
            start: curve.start + scale,
            end: curve.end + scale,
            ..curve
        };
        let progress = shifted.end + 3 * scale;
        let circulation = progress + 128 * scale;
        let backing = |curve: Curve| {
            curve.primitive(progress, &s)
                + 2u32 * big(curve.floor) * big(circulation - progress) * &s * &s
        };
        let ideal = outward(backing(shifted) - backing(curve), &area_denominator);
        let state =
            MarketState::new(segmentation, Accounting::new(nn(progress), nn(circulation))).unwrap();
        let raised = AvmCalculator::new(scalar)
            .verify_floor_raise(
                &state,
                FloorRaiseProposal {
                    floor_increase_by: pos(shifted.floor - curve.floor),
                    ramp_end: nn(shifted.end),
                    ramp_width: nn(shifted.end - shifted.start),
                },
                FloorRaisePolicy::new(nn(8), from_raw(1024 * scale)),
                &engine,
            )
            .unwrap();
        assert_eq!(
            raised.next_state.segmentation().main_offset(),
            from_raw(offset)
        );
        let raised_calculator = ScheduleCalculator::new(raised.next_state.segmentation(), scalar);
        assert_eq!(
            calculator.y(nn(progress + 1), &engine).unwrap(),
            raised_calculator.y(nn(progress + 1), &engine).unwrap(),
        );
        assert_eq!(
            calculator
                .integrate(nn(progress + 1), nn(scale + 1), &engine)
                .unwrap(),
            raised_calculator
                .integrate(nn(progress + 1), nn(scale + 1), &engine)
                .unwrap(),
        );
        // Bound accumulated uncertainty in the residual independently of how
        // the verifier divides the affected interval into characteristic spans.
        check_bounds(
            (
                to_raw(raised.backing_change.lower()),
                to_raw(raised.backing_change.upper()),
            ),
            ideal,
            64,
        );
    }
}

#[test]
fn segmented_calculations_enclose_exact_oracles_on_three_grids() {
    check_grid(
        1i128 << 64,
        I64F64::from_scaled_i128,
        I64F64::to_scaled_i128,
    );
    check_grid(
        1_000_000_000_000,
        DashuFixed12::from_scaled_i128,
        DashuFixed12::to_scaled_i128,
    );
    check_grid(
        10,
        |raw| I64Scaled::<10>::from_scaled_i64(raw.try_into().unwrap()),
        |number| number.to_scaled_i64().into(),
    );
}

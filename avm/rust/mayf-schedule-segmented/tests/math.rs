//! Independent raw-integer oracles for affine composition and piecewise areas.
use core::fmt::Debug;
use dashu::{base::DivRem, integer::UBig};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_drivers::{I64F64, I64Scaled};
use mayf_fixed_number::{AboveOne, Bounds, NonNeg, Pos, SignedFixed};
use mayf_schedule_segmented::{
    MainReference, Segmentation, ShiftedScheduleCalculator, compile_segmented_schedule,
};

#[derive(Clone, Copy, Debug)]
struct Curve {
    floor: i128,
    slope: i128,
    scalar: i128,
    start: i128,
    end: i128,
    shift: i128,
}

fn big(value: i128) -> UBig {
    UBig::from(u128::try_from(value).unwrap())
}

impl Curve {
    // Price in raw units, with denominator S^2.
    fn price(self, x: i128, s: &UBig) -> UBig {
        let ramp = (x - self.shift - self.start)
            .max(0)
            .min(self.end - self.start);
        let main = (x - self.shift - self.end).max(0);
        big(self.floor) * s * s
            + big(self.slope) * big(self.scalar) * big(ramp)
            + big(self.slope) * s * big(main)
    }

    // Integral from zero in raw units, with denominator 2*S^3.
    // Positive-part squares account for both joins independently of splitting.
    fn primitive(self, x: i128, s: &UBig) -> UBig {
        let start_distance = big((x - self.shift - self.start).max(0));
        let end_distance = big((x - self.shift - self.end).max(0));
        let start_square = &start_distance * &start_distance;
        let end_square = &end_distance * &end_distance;
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

fn assert_encloses(actual: (i128, i128), ideal: (i128, i128), slack: i128) {
    assert!(
        actual.0 <= ideal.0 && ideal.1 <= actual.1,
        "{actual:?} excludes {ideal:?}"
    );
    assert!(
        ideal.0 - actual.0 <= slack && actual.1 - ideal.1 <= slack,
        "{actual:?} is too loose around {ideal:?}, allowed {slack} quanta"
    );
}

fn check_math<N: SignedFixed + Debug>(scale: i128, from_raw: fn(i128) -> N, to_raw: fn(N) -> i128) {
    let nn = |raw| NonNeg::new(from_raw(raw)).unwrap();
    let pos = |raw| Pos::new(from_raw(raw)).unwrap();
    let edges = |b: Bounds<NonNeg<N>>| (to_raw(b.lower().get()), to_raw(b.upper().get()));
    let s = big(scale);
    let price_denominator = &s * &s;
    let area_denominator = 2u32 * &s * &s * &s;
    let mut state = 0x6d61_7966_7365_676du128;
    let mut sample = |limit: i128| {
        state = state
            .wrapping_mul(0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645)
            .wrapping_add(1);
        (state % (limit as u128)) as i128
    };
    for case in 0..128 {
        let mut curve = Curve {
            floor: sample(8 * scale),
            slope: 1 + sample(4 * scale),
            scalar: scale + 1 + sample(2 * scale),
            start: if case % 16 == 0 { 0 } else { sample(4 * scale) },
            end: 0,
            shift: 0,
        };
        curve.end = curve.start + if case % 8 == 0 { 0 } else { sample(4 * scale) };
        curve.shift = if case % 4 == 0 {
            -curve.start
        } else {
            sample(curve.start + 3 * scale + 1) - curve.start
        };
        let unshifted = Curve { shift: 0, ..curve };
        // Certify the exact ramp-end rate independently, including off-grid
        // rates whose reference must retain a nonzero enclosure width.
        let (lower, upper) = outward(unshifted.price(curve.end, &s), &price_denominator);
        let segmentation = Segmentation::new(
            nn(curve.floor),
            nn(curve.end),
            nn(curve.end - curve.start),
            AboveOne::new(from_raw(curve.scalar)).unwrap(),
            MainReference::new(nn(curve.end), Bounds::new(nn(lower), nn(upper)).unwrap()),
        )
        .unwrap();
        let characteristic = LinearCharacteristic::new(pos(curve.slope));
        let core = compile_segmented_schedule(&characteristic, segmentation).unwrap();
        let shifted = ShiftedScheduleCalculator::new(core, from_raw(curve.shift)).unwrap();

        for (expected, use_shift) in [(unshifted, false), (curve, true)] {
            let start = expected.start + expected.shift;
            let end = expected.end + expected.shift;
            for x in [
                0,
                (start - 1).max(0),
                start,
                start + 1,
                (end - 1).max(0),
                end,
                end + 1,
                end + 3 * scale,
            ] {
                let price = if use_shift {
                    shifted.y(nn(x))
                } else {
                    shifted.inner().y(nn(x))
                }
                .unwrap();
                // Ramp slope rounding contributes < 4q, with < 2q final
                // rounding. The certified main anchor is at most 1q wide.
                assert_encloses(
                    edges(price),
                    outward(expected.price(x, &s), &price_denominator),
                    6,
                );
            }
            for (lo, width) in [
                (0, end + 3 * scale),
                ((start - 1).max(0), 2),
                ((end - 1).max(0), 2),
                (start, end - start),
                (end, 3 * scale),
                (end + 1, 1),
            ] {
                if width == 0 {
                    continue;
                }
                let area = if use_shift {
                    shifted.integrate(nn(lo), pos(width))
                } else {
                    shifted.inner().integrate(nn(lo), pos(width))
                }
                .unwrap();
                let ideal = outward(
                    expected.primitive(lo + width, &s) - expected.primitive(lo, &s),
                    &area_denominator,
                );
                // Floor: <= 1q. Ramp: < 8q slope error plus <= 5q formula
                // rounding. Main: <= 3q reference error plus <= 5q rounding.
                assert_encloses(edges(area), ideal, 24);
            }
        }

        // A contraction preserves the retained prefix and its enclosure, while
        // replacing future amplified growth with unscaled characteristic growth.
        let original = shifted.inner();
        for remaining in [
            0,
            curve.start,
            (curve.start + curve.end) / 2,
            curve.end,
            curve.end + scale,
        ] {
            let next = original.contract_to(nn(remaining)).unwrap();
            let expected = Curve {
                start: curve.start.min(remaining),
                end: curve.end.min(remaining),
                ..unshifted
            };
            assert_eq!(to_raw(next.ramp_end().get()), expected.end);
            assert_eq!(to_raw(next.ramp_start().unwrap().get()), expected.start);
            if remaining < curve.end {
                assert_eq!(next.main_reference().curve_x(), nn(remaining));
            }
            let contracted = compile_segmented_schedule(&characteristic, next).unwrap();
            assert_eq!(
                contracted.y(nn(remaining)).unwrap(),
                original.y(nn(remaining)).unwrap()
            );
            for x in [0, remaining, curve.end + scale] {
                assert_encloses(
                    edges(contracted.y(nn(x)).unwrap()),
                    outward(expected.price(x, &s), &price_denominator),
                    6,
                );
            }
            if remaining > 0 {
                assert_eq!(
                    contracted.integrate(nn(0), pos(remaining)).unwrap(),
                    original.integrate(nn(0), pos(remaining)).unwrap()
                );
            }
        }
    }
}

#[test]
fn affine_prices_areas_contraction_and_signed_shifts_enclose_exact_oracles() {
    check_math(
        1i128 << 64,
        I64F64::from_scaled_i128,
        I64F64::to_scaled_i128,
    );
    check_math(
        1_000_000_000_000,
        DashuFixed12::from_scaled_i128,
        DashuFixed12::to_scaled_i128,
    );
    // A coarse decimal grid exposes rounding that is tiny on production grids.
    check_math(
        10,
        |raw| I64Scaled::<10>::from_scaled_i64(raw.try_into().unwrap()),
        |n| n.to_scaled_i64().into(),
    );
}

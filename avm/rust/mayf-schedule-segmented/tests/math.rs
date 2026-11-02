//! Exact raw-integer oracles for composition, independent of driver rounding.
use core::fmt::Debug;
use dashu::{base::DivRem, integer::UBig};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_drivers::{I64F64, I64Scaled};
use mayf_fixed_number::{AboveOne, Bounds, NonNeg, Pos, SignedFixed};
use mayf_schedule_segmented::{
    Schedule, Segmentation, SegmentedRaiseFloorProposal, SegmentedSchedule,
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
    // Price numerator with denominator S^2.
    fn price(self, x: i128, s: &UBig) -> UBig {
        let ramp = (x - self.shift - self.start)
            .max(0)
            .min(self.end - self.start);
        let main = (x - self.shift - self.end).max(0);
        big(self.floor) * s * s
            + big(self.slope) * big(self.scalar) * big(ramp)
            + big(self.slope) * s * big(main)
    }

    // Integral from zero, numerator with denominator 2*S^3. Positive-part
    // squares account for both boundaries without calling piecewise calculators.
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
            slope: scale + sample(3 * scale),
            scalar: scale + 1 + sample(2 * scale),
            start: sample(4 * scale),
            end: 0,
            shift: sample(3 * scale),
        };
        curve.end = curve.start + if case % 8 == 0 { 0 } else { sample(4 * scale) };
        let geometry = Segmentation::new(
            nn(curve.floor),
            nn(curve.start),
            nn(curve.end),
            AboveOne::new(from_raw(curve.scalar)).unwrap(),
        )
        .unwrap();
        let schedule = SegmentedSchedule::with_horizontal_shift(
            LinearCharacteristic::new(Pos::new(from_raw(curve.slope)).unwrap()),
            geometry,
            nn(curve.shift),
        )
        .unwrap();
        let start = curve.shift + curve.start;
        let end = curve.shift + curve.end;
        // Sample both sides and the exact position of every join.
        for x in [
            0,
            curve.shift,
            (start - 1).max(0),
            start,
            start + 1,
            (end - 1).max(0),
            end,
            end + 1,
            end + 3 * scale,
        ] {
            assert_encloses(
                edges(schedule.y(nn(x)).unwrap()),
                outward(curve.price(x, &s), &price_denominator),
                6,
            );
        }
        for (lo, width) in [
            (0, end + 3 * scale),
            ((start - 1).max(0), 2),
            ((end - 1).max(0), 2),
            (start, end - start),
            (end, 3 * scale),
            (0, 0),
        ] {
            let hi = lo + width;
            let ideal = outward(
                curve.primitive(hi, &s) - curve.primitive(lo, &s),
                &area_denominator,
            );
            // Floor rounding <= 1q; ramp slope error times span <= 8q, plus
            // <= 5q formula rounding; main anchor error <= 6q over width <= 4,
            // plus <= 4q formula rounding. All three grids have q <= 0.1.
            let area = schedule.integrate(nn(lo), nn(width)).unwrap();
            assert_encloses(edges(area), ideal, 1 + 8 + 5 + 6 * 4 + 4);
            if lo >= curve.shift {
                let buy = schedule
                    .quote_buy_with_exact_ava_out(nn(lo), nn(width))
                    .unwrap();
                let sell = schedule
                    .quote_sell_with_exact_ava_in(nn(hi), nn(width))
                    .unwrap();
                assert_eq!(buy.reserve_in, area.upper());
                assert_eq!(sell.reserve_out, area.lower());
            }
        }

        // Contract into each segment, preserving the part already backing supply.
        for supply in [
            0,
            curve.start,
            (curve.start + curve.end) / 2,
            curve.end,
            curve.end + scale,
        ] {
            let contracted = schedule.contract_to(nn(supply + curve.shift)).unwrap();
            let expected = Curve {
                start: curve.start.min(supply),
                end: curve.end.min(supply),
                ..curve
            };
            for x in [curve.shift, supply + curve.shift, end + scale] {
                assert_encloses(
                    edges(contracted.y(nn(x)).unwrap()),
                    outward(expected.price(x, &s), &price_denominator),
                    6,
                );
            }
        }

        let proposal = SegmentedRaiseFloorProposal::new(Pos::new(from_raw(scale)).unwrap(), nn(0));
        let new_start = curve.start + scale;
        let new_end = curve.end.max(new_start);
        let raised = schedule
            .raise_floor(nn(curve.shift + new_end + scale), proposal)
            .unwrap();
        // Independently certify k*m*advance - (k-1)*m*end_advance.
        let ideal_increase = outward(
            big(curve.slope)
                * (big(curve.scalar) * &s - big(curve.scalar - scale) * big(new_end - curve.end)),
            &price_denominator,
        );
        let increase = to_raw(raised.new_floor.get()) - curve.floor;
        let loss = to_raw(raised.max_live_price_decrease.get());
        assert_encloses((increase, increase + loss), ideal_increase, 7);
        let expected = Curve {
            floor: curve.floor + increase,
            start: new_start,
            end: new_end,
            ..curve
        };
        let future = curve.shift + new_end + scale;
        assert_encloses(
            edges(raised.next_schedule.y(nn(future)).unwrap()),
            outward(expected.price(future, &s), &price_denominator),
            6,
        );
        assert_encloses(
            edges(raised.next_schedule.required_reserve(nn(future)).unwrap()),
            outward(expected.primitive(future, &s), &area_denominator),
            1 + 8 + 5 + 6 * 4 + 4,
        );
    }
}

#[test]
fn segmented_points_areas_and_transitions_enclose_exact_oracles_on_three_grids() {
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
    check_math(
        10,
        |raw| I64Scaled::<10>::from_scaled_i64(raw.try_into().unwrap()),
        |n| n.to_scaled_i64().into(),
    );
}

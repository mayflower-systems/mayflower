//! Raw-integer oracles independent of the calculator's interval splitting.
use core::fmt::Debug;

use dashu::{base::DivRem, integer::UBig};
use mayf_avm_calculator::{Segmentation, compile_schedule};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_drivers::{I64F64, I64Scaled};
use mayf_fixed_number::{AboveOne, Bounds, NonNeg, Pos, SignedFixed};

#[derive(Clone, Copy)]
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
        let start = if case % 16 == 0 { 0 } else { sample(4 * scale) };
        let curve = Curve {
            floor: 1 + sample(8 * scale),
            slope: 1 + sample(4 * scale),
            scalar: scale + 1 + sample(2 * scale),
            start,
            end: start + if case % 8 == 0 { 0 } else { sample(4 * scale) },
        };
        let segmentation =
            Segmentation::new(pos(curve.floor), nn(curve.end), nn(curve.end - curve.start))
                .unwrap();
        let characteristic = LinearCharacteristic::new(pos(curve.slope));
        let scalar = AboveOne::new(from_raw(curve.scalar)).unwrap();
        let calculator = compile_schedule(&characteristic, scalar, segmentation).unwrap();
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
            // A <=4-unit ramp amplifies slope rounding by <4q. Its derived
            // endpoint and the main multiplication each add at most one quantum.
            check_bounds(
                edges(calculator.y(nn(x)).unwrap()),
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
            // Floor <=1q; ramp slope error <8q plus final rounding; derived
            // main-anchor error integrated over <=3 units contributes <15q.
            check_bounds(
                edges(calculator.integrate(nn(lo), nn(width)).unwrap()),
                ideal,
                32,
            );
        }

        for remaining in [
            0,
            curve.start,
            (curve.start + curve.end) / 2,
            curve.end,
            curve.end + scale,
        ] {
            let contracted_state = segmentation.contract_to(nn(remaining)).unwrap();
            let expected = Curve {
                start: curve.start.min(remaining),
                end: curve.end.min(remaining),
                ..curve
            };
            let contracted = compile_schedule(&characteristic, scalar, contracted_state).unwrap();
            assert_eq!(
                contracted.y(nn(remaining)).unwrap(),
                calculator.y(nn(remaining)).unwrap()
            );
            assert_eq!(
                contracted.integrate(nn(0), nn(remaining)).unwrap(),
                calculator.integrate(nn(0), nn(remaining)).unwrap()
            );
            for x in [remaining, curve.end + scale] {
                check_bounds(
                    edges(contracted.y(nn(x)).unwrap()),
                    outward(expected.price(x, &s), &price_denominator),
                    8,
                );
            }
        }
    }
}

#[test]
fn prices_integrals_and_contracted_prefixes_enclose_exact_oracles_on_three_grids() {
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

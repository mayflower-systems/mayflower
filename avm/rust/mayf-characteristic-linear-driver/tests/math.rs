//! Exact integer oracles: no floating point or production fixed-point arithmetic
//! is used to compute expected values.
use core::fmt::Debug;
use dashu::{base::DivRem, integer::UBig};
use mayf_characteristic::{AffineAnchor, AffineTransform, Characteristic};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_drivers::{I64F64, I64Scaled};
use mayf_fixed_number::{Bounds, NonNeg, Pos, SignedFixed};
use mayf_monotone::MonotoneCalculator;

fn outward(numerator: UBig, denominator: &UBig) -> (i128, i128) {
    let (floor, remainder) = numerator.div_rem(denominator);
    let floor = i128::try_from(floor).unwrap();
    (floor, floor + i128::from(remainder != UBig::ZERO))
}

fn assert_encloses(actual: (i128, i128), ideal: (i128, i128), slack: i128) {
    assert!(
        actual.0 <= ideal.0 && actual.1 >= ideal.1,
        "{actual:?} excludes {ideal:?}"
    );
    assert!(
        ideal.0 - actual.0 <= slack && actual.1 - ideal.1 <= slack,
        "{actual:?} is too loose around {ideal:?} (slack {slack} quanta)"
    );
}

fn check_math<N: SignedFixed + Debug>(scale: i128, from_raw: fn(i128) -> N, to_raw: fn(N) -> i128) {
    let nn = |raw| NonNeg::new(from_raw(raw)).unwrap();
    let edges =
        |bounds: Bounds<NonNeg<N>>| (to_raw(bounds.lower().get()), to_raw(bounds.upper().get()));
    let s = UBig::from(scale as u128);
    let s3 = &s * &s * &s;
    let area_denominator = 2u32 * &s3 * &s;
    let mut state = 0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645u128;
    let mut sample = |limit: i128| {
        state = state.wrapping_mul(0xda94_2042_e4dd_58b5).wrapping_add(1);
        (state % (limit as u128)) as i128
    };

    for case in 0..256 {
        let m = if case % 64 == 0 {
            (1i128 << 50) * scale + sample(scale)
        } else {
            1 + sample(16 * scale)
        };
        let b = scale / 2 + sample(3 * scale);
        let a = sample(4 * scale);
        let anchor_x = sample(8 * scale);
        let anchor_y = sample(16 * scale);
        let uncertainty = sample(4);
        let lo = anchor_x + sample(8 * scale);
        let width = 1 + sample(4 * scale);
        let hi = lo + width;
        // b >= 1/2 makes this negative translation valid at the anchor.
        let c = if case % 2 == 0 {
            -anchor_x / 4
        } else {
            3 * scale
        };
        let anchor = AffineAnchor::new(
            nn(anchor_x),
            Bounds::new(nn(anchor_y), nn(anchor_y + uncertainty)).unwrap(),
        );
        let make = |translation| {
            LinearCharacteristic::new(Pos::new(from_raw(m)).unwrap()).calculator(
                AffineTransform::new(
                    Pos::new(from_raw(b)).unwrap(),
                    from_raw(translation),
                    nn(a),
                    anchor,
                ),
            )
        };
        let calc = make(c);
        let translated = make(5 * scale);
        let coefficient = UBig::from(m as u128) * UBig::from(b as u128) * UBig::from(a as u128);

        for x in [anchor_x, lo, hi] {
            let delta = &coefficient * UBig::from((x - anchor_x) as u128);
            let lower = outward(UBig::from(anchor_y as u128) * &s3 + &delta, &s3).0;
            let upper = outward(
                UBig::from((anchor_y + uncertainty) as u128) * &s3 + &delta,
                &s3,
            )
            .1;
            let actual = calc.y(nn(x)).unwrap();
            // Two slope products introduce < (a+1) quanta of error;
            // distance < 12 and a < 4, then one final product rounds.
            assert_encloses(edges(actual), (lower, upper), 5 * 12 + 1);
            assert_eq!(
                actual,
                translated.y(nn(x)).unwrap(),
                "translation case {case}"
            );
        }

        // Independently integrate the quadratic primitive, in raw integers:
        // anchor_y*w + m*b*a*((hi-anchor_x)^2-(lo-anchor_x)^2)/2.
        let lo_distance = UBig::from((lo - anchor_x) as u128);
        let hi_distance = UBig::from((hi - anchor_x) as u128);
        let delta_area = &coefficient * (&hi_distance * &hi_distance - &lo_distance * &lo_distance);
        let anchor_area = |y| UBig::from(y as u128) * UBig::from(width as u128) * 2u32 * &s3;
        let lower = outward(anchor_area(anchor_y) + &delta_area, &area_denominator).0;
        let upper = outward(
            anchor_area(anchor_y + uncertainty) + &delta_area,
            &area_denominator,
        )
        .1;
        let actual = calc.integrate(nn(lo), nn(width)).unwrap();
        // Slope error < 5 quanta times span < 40, plus <= 5 quanta
        // from integrating the whole/fractional parts and anchor rectangle.
        assert_encloses(edges(actual), (lower, upper), 5 * 40 + 5);
        assert_eq!(actual, translated.integrate(nn(lo), nn(width)).unwrap());
    }
}

#[test]
fn points_and_integrals_enclose_exact_oracles_on_binary_and_decimal_grids() {
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
    // A coarse grid makes rounding errors large enough to expose sign/order mistakes.
    check_math(
        10,
        |raw| I64Scaled::<10>::from_scaled_i64(raw.try_into().unwrap()),
        |n| n.to_scaled_i64().into(),
    );
}

const SCALE: i128 = 1i128 << 64;
fn raw(value: i128) -> NonNeg<I64F64> {
    NonNeg::new(I64F64::from_scaled_i128(value)).unwrap()
}
fn whole(value: i128) -> NonNeg<I64F64> {
    raw(value * SCALE)
}

#[test]
fn fractional_anchor_does_not_amplify_rounding_of_translated_coordinates() {
    let anchor = raw(SCALE + 1);
    let calc = LinearCharacteristic::try_new(whole(1_000_000).get())
        .unwrap()
        .calculator(AffineTransform::new(
            Pos::new(raw(SCALE / 2).get()).unwrap(),
            whole(3).get(),
            whole(1),
            AffineAnchor::exact(anchor, whole(1)),
        ));
    assert_eq!(
        calc.y(raw(SCALE + 2)).unwrap(),
        Bounds::exact(raw(SCALE + 500_000))
    );
}

#[test]
fn tiny_scales_and_one_quantum_widths_retain_useful_precision() {
    let calc = LinearCharacteristic::try_new(whole(1_000_000).get())
        .unwrap()
        .calculator(AffineTransform::new(
            Pos::new(raw(1).get()).unwrap(),
            whole(3).get(),
            whole(4),
            AffineAnchor::exact(whole(0), whole(1)),
        ));
    // g(x) = 1 + 4,000,000*q*x; integral from 16 to 18 is 2 + 136,000,000*q.
    assert_eq!(
        calc.integrate(whole(16), whole(2)).unwrap(),
        Bounds::exact(raw(2 * SCALE + 136_000_000))
    );

    let calc = LinearCharacteristic::try_new(whole(686_260).get())
        .unwrap()
        .calculator(AffineTransform::new(
            Pos::one(),
            I64F64::ZERO,
            whole(1),
            AffineAnchor::exact(raw(SCALE + 1), whole(1)),
        ));
    let area = calc.integrate(whole(12), raw(1)).unwrap();
    // The exact area is (1 + 686260*11)*q - 686260*q^2/2.
    assert_encloses(
        (
            area.lower().get().to_scaled_i128(),
            area.upper().get().to_scaled_i128(),
        ),
        (7_548_860, 7_548_861),
        1,
    );
}

#[test]
fn integral_avoids_overflowing_unscaled_products() {
    let calc =
        LinearCharacteristic::new(Pos::<I64F64>::one()).calculator(AffineTransform::identity());
    // The upper endpoint rectangle overflows, while the factored integral fits.
    let lo = whole(2_000_000_000);
    let width = whole(2_500_000_000);
    assert!(
        calc.y(lo.checked_add(width).unwrap())
            .unwrap()
            .upper()
            .checked_mul_ceil(width)
            .is_err()
    );
    let area = calc.integrate(lo, width).unwrap();
    assert_eq!(area, Bounds::exact(whole(8_125_000_000_000_000_000)));

    // Exercise whole/fractional slope decomposition at the numeric range limit.
    let calc = LinearCharacteristic::try_new(I64F64::MAX)
        .unwrap()
        .calculator(AffineTransform::identity());
    assert_eq!(calc.y(whole(1)).unwrap(), Bounds::exact(raw(i128::MAX)));
    let area = calc.integrate(whole(0), raw(1)).unwrap();
    assert_encloses(
        (
            area.lower().get().to_scaled_i128(),
            area.upper().get().to_scaled_i128(),
        ),
        (0, 1),
        1,
    );
}

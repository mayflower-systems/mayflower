//! Compare fixed-point results with the exact linear primitive using integer arithmetic.
use dashu::{base::DivRem, integer::UBig};
use mayf_characteristic::{Characteristic, CharacteristicError};
use mayf_characteristic_linear_driver::LinearCharacteristic;
use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_drivers::{I64F64, I64Scaled, U64F64};
use mayf_fixed_number::{Bounds, FixedNumber, NonNeg, Pos};

fn outward(numerator: UBig, denominator: &UBig) -> (u128, u128) {
    let (floor, remainder) = numerator.div_rem(denominator);
    let floor = u128::try_from(floor).unwrap();
    (floor, floor + u128::from(remainder != UBig::ZERO))
}

fn check_grid<N: FixedNumber>(scale: u128, from_raw: fn(u128) -> N, to_raw: fn(N) -> u128) {
    let nn = |raw| NonNeg::new(from_raw(raw)).unwrap();
    let edges = |b: Bounds<NonNeg<N>>| (to_raw(b.lower().get()), to_raw(b.upper().get()));
    let s = UBig::from(scale);
    let area_denominator = 2u8 * &s * &s;

    for slope in [
        1,
        scale / 3,
        scale,
        3 * scale + 1,
        (scale << 45) + scale / 3,
    ] {
        let line = LinearCharacteristic::try_new(from_raw(slope)).unwrap();
        for (lo, width) in [
            (0, 0),
            (0, 1),
            (0, 2 * scale),
            (scale + 1, 1),
            (7 * scale / 3, 13 * scale / 4),
            (5 * scale + 1, 3 * scale + 1),
        ] {
            let hi = lo + width;
            let m = UBig::from(slope);
            let x0 = UBig::from(lo);
            let x1 = UBig::from(hi);
            assert_eq!(edges(line.y(nn(hi)).unwrap()), outward(&m * &x1, &s));

            // In raw units the exact area is m * (hi² - lo²) / (2 * scale²).
            let (floor, ceil) = outward(&m * (&x1 * &x1 - &x0 * &x0), &area_denominator);
            let (lower, upper) = edges(line.integrate(nn(lo), nn(width)).unwrap());
            assert!(
                lower <= floor && floor - lower <= 2 && upper >= ceil && upper - ceil <= 2,
                "raw slope {slope}, lo {lo}, width {width}: [{lower}, {upper}] vs [{floor}, {ceil}]"
            );
        }
    }
}

#[test]
fn point_and_integral_bounds_enclose_the_exact_values_on_each_grid() {
    check_grid(1 << 64, U64F64::from_scaled_u128, U64F64::to_scaled_u128);
    check_grid(
        1_000_000_000_000,
        |r| DashuFixed12::from_scaled_i128(r.try_into().unwrap()),
        |n| n.to_scaled_i128().try_into().unwrap(),
    );
    check_grid(
        10,
        |r| I64Scaled::<10>::from_scaled_i64(r.try_into().unwrap()),
        |n| n.to_scaled_i64().try_into().unwrap(),
    );
}

#[test]
fn integral_can_fit_when_trapezoid_intermediates_overflow() {
    let line = LinearCharacteristic::new(Pos::<I64F64>::one());
    let whole = |value| NonNeg::new(I64F64::from(value)).unwrap();
    let lo = whole(2_000_000_000_i64);
    let width = whole(2_500_000_000);
    assert!(
        lo.checked_add(width)
            .unwrap()
            .checked_mul_ceil(width)
            .is_err()
    );
    assert_eq!(
        line.integrate(lo, width).unwrap(),
        Bounds::exact(whole(8_125_000_000_000_000_000))
    );

    // Here both endpoints fit but their sum would overflow. The exact area is max − 1/2.
    let max = NonNeg::new(I64F64::MAX).unwrap();
    let lo = max.checked_sub(whole(1)).unwrap();
    let area = line.integrate(lo, whole(1)).unwrap();
    let exact = i128::MAX - (1 << 63);
    assert!((exact - 1..=exact).contains(&area.lower().get().to_scaled_i128()));
    assert!((exact..=exact + 1).contains(&area.upper().get().to_scaled_i128()));
}

#[test]
fn extreme_inputs_distinguish_tiny_areas_from_overflow() {
    let max = NonNeg::new(I64F64::MAX).unwrap();
    let line = LinearCharacteristic::try_new(max.get()).unwrap();
    let zero = NonNeg::zero();
    let two = NonNeg::from_u64(2).unwrap();
    assert_eq!(line.integrate(max, zero), Ok(Bounds::exact(zero)));
    // At maximum slope and one quantum of width the true area is under one quantum.
    let quantum = NonNeg::new(I64F64::QUANTUM).unwrap();
    let tiny = line.integrate(zero, quantum).unwrap();
    assert_eq!(tiny.lower(), zero);
    assert!((1..=2).contains(&tiny.upper().get().to_scaled_i128()));
    let overflow = Err(CharacteristicError::ArithmeticFailure);
    assert_eq!(line.y(two), overflow);
    assert_eq!(line.integrate(max, NonNeg::one()), overflow);
    assert_eq!(line.integrate(zero, two), overflow);
}

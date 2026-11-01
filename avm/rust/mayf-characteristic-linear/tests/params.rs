use mayf_characteristic_linear::{LinearParams, LinearParamsError};
use mayf_fixed_dashu::DashuFixed12;
use mayf_fixed_number::{FixedNumber, Pos};

type Fx = DashuFixed12;

fn n(value: i64) -> Fx {
    Fx::checked_from_i128(i128::from(value)).unwrap()
}

#[test]
fn stores_only_a_valid_neutral_slope() {
    let params = LinearParams::try_new(n(2)).unwrap();

    assert_eq!(params.slope().get(), n(2));
    assert_eq!(
        LinearParams::try_new(n(0)),
        Err(LinearParamsError::NonPositiveSlope)
    );
    assert_eq!(
        LinearParams::try_new(n(-1)),
        Err(LinearParamsError::NonPositiveSlope)
    );
}

#[test]
fn fuzz_linear_parameter_taxonomy() {
    let mut state: u128 = 0xbeef_cafe_1234_5678_8765_4321_abcd_ef01;
    let next_u128 = |state: &mut u128| -> u128 {
        *state = state
            .wrapping_mul(0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645)
            .wrapping_add(0x1405_7b7e_f767_814f);
        *state
    };

    for _ in 0..1000 {
        let slope_raw = (next_u128(&mut state) % 2000) as i128 - 1000;
        let slope = Fx::checked_from_i128(slope_raw).unwrap();

        let res = LinearParams::try_new(slope);
        if slope_raw <= 0 {
            assert_eq!(res, Err(LinearParamsError::NonPositiveSlope));
        } else {
            let p = res.unwrap();
            assert_eq!(p.slope().get(), slope);

            let direct = LinearParams::new(Pos::new(slope).unwrap());
            assert_eq!(p, direct);
        }
    }
}

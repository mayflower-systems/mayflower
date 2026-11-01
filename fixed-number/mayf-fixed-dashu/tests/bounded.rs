use dashu::{base::DivRem, integer::IBig};
use mayf_fixed_dashu::DashuFixed12 as N;
use mayf_fixed_number::{Bounded, FixedNumber, IntegralRounding, MathError};

#[test]
fn decimal_chains_preserve_bounds_and_fused_range() {
    let third = N::ONE.bounded() / 3;
    let payout = (Bounded::<N>::from(1_000_000_u64) * (1 - third))
        .bounds()
        .unwrap();
    assert_eq!(payout.lower().get().checked_floor_to_u64(), Ok(666_666));
    assert!(payout.upper().get() < N::checked_from_i128(666_667).unwrap());
    assert_eq!(
        (N::MAX.bounded() * 2 / 2)
            .bounds()
            .map(|b| b.into_raw().into_edges()),
        Ok((N::MAX, N::MAX))
    );
    assert_eq!((N::MAX.bounded() * 2).bounds(), Err(MathError::Overflow));
}

#[test]
fn fused_decimal_bounds_match_independent_big_integer_division() {
    let mut seed = 0x853c_49e6_748f_ea9b_da3e_39cb_94b9_5bdb_u128;
    let mut next = || {
        seed = seed
            .wrapping_mul(0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645)
            .wrapping_add(1);
        (seed >> 1) as i128
    };
    for _ in 0..2000 {
        let (a, b, d) = (next(), next(), next().max(1));
        let actual = (N::from_scaled_i128(a).bounded() * N::from_scaled_i128(b)
            / N::from_scaled_i128(d))
        .bounds()
        .map(|bounds| bounds.into_raw().into_edges())
        .map(|(lo, hi)| (lo.to_scaled_i128(), hi.to_scaled_i128()));
        let (lo, remainder) = (IBig::from(a) * IBig::from(b)).div_rem(IBig::from(d));
        let hi = if remainder == IBig::ZERO {
            lo.clone()
        } else {
            &lo + IBig::ONE
        };
        let expected = i128::try_from(lo)
            .and_then(|lo| i128::try_from(hi).map(|hi| (lo, hi)))
            .map_err(|_| MathError::Overflow);
        assert_eq!(actual, expected, "{a} * {b} / {d}");
    }
}

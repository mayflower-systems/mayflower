use mayf_fixed_drivers::{I64F64, U64F64, U192F64};
use mayf_fixed_number::{Bounded, Bounds, FixedNumber, IntegralRounding, MathError, MathResult};

fn edges<N: FixedNumber>(value: Bounded<N>) -> MathResult<(N, N)> {
    value.bounds().map(|bounds| bounds.into_raw().into_edges())
}

fn common<N: IntegralRounding + core::fmt::Debug>() {
    let third = N::ONE.bounded() / 3;
    let amount = (third * 3).bounds().unwrap();
    assert!(amount.contains(mayf_fixed_number::NonNeg::one()));
    assert_eq!(amount.lower().get().checked_floor_to_u64(), Ok(0));
    let tiny = Bounds::new(N::ZERO, N::QUANTUM)
        .unwrap()
        .try_nonnegative()
        .unwrap()
        .bounded();
    assert_eq!(edges(N::ONE.bounded() / tiny), Err(MathError::Domain));
    let payout = (Bounded::<N>::from(1_000_000_u64) * (1 - third))
        .bounds()
        .unwrap();
    assert_eq!(payout.lower().get().checked_floor_to_u64(), Ok(666_666));
    assert!(payout.upper().get() < N::checked_from_i128(666_667).unwrap());
}

#[test]
fn binary_drivers_share_nonnegative_expression_contracts() {
    common::<I64F64>();
    common::<U64F64>();
    common::<U192F64>();
}

#[test]
fn entire_unsigned_storage_range_is_usable_without_losing_upper_overflow() {
    let maximum = U64F64::MAX.bounded();
    assert_eq!(edges(maximum * 2 / 2), Ok((U64F64::MAX, U64F64::MAX)));
    assert_eq!(edges(maximum * 2), Err(MathError::Overflow));
    assert_eq!(
        maximum
            .bounds()
            .unwrap()
            .lower()
            .get()
            .checked_floor_to_u64(),
        Ok(u64::MAX)
    );
}

#[test]
fn expression_storage_is_fixed_size_and_requires_no_allocation() {
    // Layout is diagnostic, not a two-endpoint-size promise.
    eprintln!(
        "Bounded<I64F64>: {} bytes; Bounded<U192F64>: {} bytes",
        core::mem::size_of::<Bounded<I64F64>>(),
        core::mem::size_of::<Bounded<U192F64>>()
    );
    assert!(core::mem::size_of::<Bounded<I64F64>>() <= 96);
    assert!(core::mem::size_of::<Bounded<U192F64>>() <= 160);
}

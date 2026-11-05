//! Checked fixed-number helpers shared by the actions.

use crate::{EpochDuration, Result, RewardClearingError};
use core::num::{NonZeroU64, NonZeroU128};
use mayf_fixed_number::{DirectedRounding, FixedNumber, MathResult, NonNeg, Pos};

/// Map any numeric driver failure to the domain's arithmetic error.
pub(crate) fn arithmetic<T>(result: MathResult<T>) -> Result<T> {
    result.map_err(|_| RewardClearingError::ArithmeticFailure)
}

pub(crate) fn positive<N: FixedNumber>(value: NonNeg<N>) -> Result<Pos<N>> {
    arithmetic(Pos::try_from(value))
}

/// Compute `numerator / duration` with the given rounding direction.
pub(crate) fn ratio<N: FixedNumber>(
    numerator: u64,
    denominator: EpochDuration,
    rounding: DirectedRounding,
) -> Result<NonNeg<N>> {
    let denominator = NonZeroU128::from(NonZeroU64::from(denominator));
    let value = arithmetic(N::checked_from_ratio_round(
        i128::from(numerator),
        denominator,
        rounding,
    ))?;
    arithmetic(NonNeg::new(value))
}

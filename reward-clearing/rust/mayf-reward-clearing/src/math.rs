//! Fixed-number helpers shared by the models.

use core::num::{NonZeroU64, NonZeroU128};
use mayf_fixed_number::{DirectedRounding, FixedNumber, MathResult, NonNeg};

/// Compute `numerator / denominator` with the given rounding direction.
pub(crate) fn ratio<N: FixedNumber>(
    numerator: u64,
    denominator: NonZeroU64,
    rounding: DirectedRounding,
) -> MathResult<NonNeg<N>> {
    let denominator = NonZeroU128::from(denominator);
    let value = N::checked_from_ratio_round(i128::from(numerator), denominator, rounding)?;
    NonNeg::new(value)
}

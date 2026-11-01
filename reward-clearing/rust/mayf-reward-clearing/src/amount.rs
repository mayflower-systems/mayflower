//! Whole accounting units and internal widening for share calculations.

use crate::{Result, RewardClearingError};
use mayf_fixed_number::{FixedNumber, NonNeg};

/// A quantity of principal or reward in indivisible accounting units.
/// The consumer chooses the unit; fractional units are never stored or carried.
pub type Amount = u64;

pub(crate) fn from_amount<N: FixedNumber>(amount: Amount) -> Result<NonNeg<N>> {
    NonNeg::from_u64(amount).map_err(|_| RewardClearingError::ArithmeticFailure)
}

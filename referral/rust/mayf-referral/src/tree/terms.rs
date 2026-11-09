use mayf_fixed_number::{FixedNumber, MathError, MathResult};
use mayf_micro_basis_points::{self as micro_basis_points, MicroBasisPoints};

use super::Depth;
use crate::{Error, Result};

/// Immutable maximum depth and gross commission for each earning ancestor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkTerms {
    max_depth: Depth,
    override_rate: MicroBasisPoints,
}

impl NetworkTerms {
    /// The deepest member retains a strictly positive part of its own yield.
    /// Depth zero permits only top-level seats; rate zero disables commissions.
    /// The library imposes no depth policy beyond the `Depth` representation.
    pub fn new(max_depth: Depth, override_rate: MicroBasisPoints) -> Result<Self> {
        if u32::from(max_depth) * override_rate.get() >= micro_basis_points::ONE {
            return Err(Error::OverrideRateTooLargeForDepth);
        }
        Ok(Self {
            max_depth,
            override_rate,
        })
    }

    pub fn max_depth(&self) -> Depth {
        self.max_depth
    }

    pub fn override_rate(&self) -> MicroBasisPoints {
        self.override_rate
    }

    /// The exact integer numerator of `1 - depth * q`; no conversion of `q`
    /// onto the accounting number's binary or decimal grid is required.
    pub(super) fn own_rate(&self, depth: Depth) -> Result<u32> {
        if depth > self.max_depth {
            return Err(Error::InvalidDepth);
        }
        Ok(micro_basis_points::ONE - u32::from(depth) * self.override_rate.get())
    }
}

/// Apply an exact micro-basis-point numerator with a retained division carry.
///
/// `carry` is measured in numerator units: its credit value is `carry / ONE`.
/// After each call it is less than `ONE * N::QUANTUM`. It belongs to the same
/// recipient or eligibility interval as every amount passed through it.
///
/// Splitting the amount before multiplying avoids the overflowing intermediate
/// `amount * numerator`; the large product is bounded by `amount`, and only a
/// remainder smaller than `ONE` accounting quanta is multiplied by the rate.
pub(super) fn apply_rate<N: FixedNumber>(
    amount: N,
    numerator: u32,
    carry: &mut N,
) -> MathResult<N> {
    if amount < N::ZERO || *carry < N::ZERO || numerator > micro_basis_points::ONE {
        return Err(MathError::Domain);
    }
    let denominator = N::checked_from_u64(u64::from(micro_basis_points::ONE))?;
    let rate = N::checked_from_u64(u64::from(numerator))?;
    let quotient = amount.checked_div_floor(denominator)?;
    let remainder = amount.checked_sub(quotient.checked_mul_floor(denominator)?)?;
    let numerator_remainder = remainder.checked_mul_floor(rate)?.checked_add(*carry)?;
    let extra = numerator_remainder.checked_div_floor(denominator)?;
    let next_carry = numerator_remainder.checked_sub(extra.checked_mul_floor(denominator)?)?;
    let result = quotient.checked_mul_floor(rate)?.checked_add(extra)?;
    *carry = next_carry;
    Ok(result)
}

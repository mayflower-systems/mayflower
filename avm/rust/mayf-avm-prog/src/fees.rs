use mayf_fixed_number::{FixedNumber, MathResult, NonNeg, Pos};
use mayf_micro_basis_points::MicroBasisPoints;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeeChain<N: FixedNumber> {
    pub fee_to_market_owner: NonNeg<N>,
    pub fee_to_protocol: NonNeg<N>,
}

impl<N: FixedNumber> Default for FeeChain<N> {
    fn default() -> Self {
        Self {
            fee_to_market_owner: NonNeg::zero(),
            fee_to_protocol: NonNeg::zero(),
        }
    }
}

/// Apply a fee with one upward rounding to the number type's native grid.
/// The integer rate and its denominator must be representable by that type.
pub fn apply_fee<N: FixedNumber>(
    amount: NonNeg<N>,
    rate: MicroBasisPoints,
) -> MathResult<NonNeg<N>> {
    let numerator = NonNeg::from_u64(u64::from(rate.get()))?;
    let denominator = Pos::from_i32(mayf_micro_basis_points::ONE as i32)?;
    amount.checked_mul_div_ceil(numerator, denominator)
}

/// Split a collected fee, rounding the protocol's share up and retaining the
/// exact remainder for the market owner.
pub fn split_fee<N: FixedNumber>(
    gross_fee_amount: NonNeg<N>,
    protocol_fee_rate: MicroBasisPoints,
) -> MathResult<FeeChain<N>> {
    let fee_to_protocol = apply_fee(gross_fee_amount, protocol_fee_rate)?;

    Ok(FeeChain {
        fee_to_protocol,
        fee_to_market_owner: gross_fee_amount.checked_sub(fee_to_protocol)?,
    })
}

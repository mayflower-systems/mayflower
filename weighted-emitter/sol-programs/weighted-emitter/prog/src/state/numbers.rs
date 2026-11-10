use anchor_lang::prelude::*;
use mayf_fixed_drivers::{U64F64, U192F64};
use mayf_fixed_number::NonNeg;

use crate::error::WeightedEmitterProgramError;

pub type Number = U192F64;

pub fn rate_from_wire(value: u128) -> NonNeg<Number> {
    NonNeg::from_unsigned(U64F64::from_scaled_u128(value).into())
}

pub fn rate_to_wire(value: NonNeg<Number>) -> Result<u128> {
    U64F64::try_from(value.get())
        .map(U64F64::to_scaled_u128)
        .map_err(|_| error!(WeightedEmitterProgramError::ArithmeticOverflow))
}

pub fn index_from_wire(value: [u8; 32]) -> NonNeg<Number> {
    NonNeg::from_unsigned(Number::from_le_bytes(value))
}

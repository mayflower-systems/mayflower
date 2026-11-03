use mayf_fixed_number::{IntegralRounding, NonNeg};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionError {
    ArithmeticFailure,
}

pub struct Position {
    /// How much AVA the position owns
    pub ava_owned: u64,
    /// How much of the reserve has been advanced
    pub reserve_advanced: u64,
}

impl Position {
    /// Total advance limit, including outstanding advances, rounded down.
    /// Returns an error if the value cannot be represented by the driver or a `u64`.
    pub fn reserve_advance_capacity<N: IntegralRounding>(
        &self,
        floor_redemption_rate: NonNeg<N>,
    ) -> Result<u64, PositionError> {
        N::checked_from_u64(self.ava_owned)
            .and_then(|amount| amount.checked_mul_floor(floor_redemption_rate.get()))
            .and_then(N::checked_floor_to_u64)
            .map_err(|_| PositionError::ArithmeticFailure)
    }

    pub fn is_healthy<N: IntegralRounding>(
        &self,
        floor_redemption_rate: NonNeg<N>,
    ) -> Result<bool, PositionError> {
        let capacity = self.reserve_advance_capacity(floor_redemption_rate)?;
        Ok(self.reserve_advanced <= capacity)
    }
}

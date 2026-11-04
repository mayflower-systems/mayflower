use mayf_fixed_number::{FixedNumber, NonNeg};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionError {
    ArithmeticFailure,
}

/// Position balances in the same normalized units as the market.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position<N: FixedNumber> {
    pub ava_owned: NonNeg<N>,
    pub reserve_advanced: NonNeg<N>,
}

impl<N: FixedNumber> Position<N> {
    /// Total advance limit, rounded down to the number type's native grid.
    pub fn reserve_advance_capacity(
        &self,
        floor_redemption_rate: NonNeg<N>,
    ) -> Result<NonNeg<N>, PositionError> {
        self.ava_owned
            .checked_mul_floor(floor_redemption_rate)
            .map_err(|_| PositionError::ArithmeticFailure)
    }

    pub fn is_healthy(&self, floor_redemption_rate: NonNeg<N>) -> Result<bool, PositionError> {
        let capacity = self.reserve_advance_capacity(floor_redemption_rate)?;
        Ok(self.reserve_advanced <= capacity)
    }
}

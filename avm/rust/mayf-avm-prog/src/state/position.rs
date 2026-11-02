use mayf_fixed_number::{FixedNumber, NonNeg};

pub struct Position {
    /// How much AVA the position owns
    pub ava_owned: u64,
    /// How much of the reserve has been advanced
    pub reserve_advanced: u64,
}

impl Position {
    /// How much reserve can be advanced given the floor redemption rate
    pub fn reserve_advance_capacity<N: FixedNumber>(
        &self,
        floor_redemption_rate: NonNeg<N>,
    ) -> u64 {
        // reserve capacity = ava_owned * floor_redemption_rate rounded down to nearest integer (defensive)
        todo!()
    }

    pub fn is_healthy<N: FixedNumber>(&self, floor_redemption_rate: NonNeg<N>) -> bool {
        let capacity = self.reserve_advance_capacity(floor_redemption_rate);
        self.reserve_advanced <= capacity
    }
}

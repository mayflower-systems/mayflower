use mayf_fixed_number::{FixedNumber, MathResult};

use crate::Amount;
use crate::numbers::Change;

/// A lane's pool: its total balance, and the credits paid per unit of balance
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanePool<N> {
    /// The sum of every member's balance in the lane
    pub balance: Amount,
    /// Credits paid per unit of balance since the lane began
    pub reward_index: N,
    /// Credits received that were too small to add to the index at this balance
    pub remainder: N,
}

impl<N: FixedNumber> Default for LanePool<N> {
    fn default() -> Self {
        Self {
            balance: 0,
            reward_index: N::ZERO,
            remainder: N::ZERO,
        }
    }
}

impl<N: FixedNumber> LanePool<N> {
    /// Pay `allocation` across the pool's balance. While no one holds the lane, it goes
    /// unallocated.
    pub(crate) fn absorb(mut self, allocation: N) -> MathResult<Self> {
        if self.balance != 0 {
            let balance = N::checked_from_u64(self.balance)?;
            let total = self.remainder.checked_add(allocation)?;
            let increment = total.checked_div_floor(balance)?;
            self.reward_index = self.reward_index.checked_add(increment)?;
            self.remainder = total.checked_sub(increment.checked_mul_floor(balance)?)?;
        }
        Ok(self)
    }

    /// Add a member's deposit to the lane's balance.
    pub(crate) fn deposit(self, amount: Amount) -> Self {
        self.change_balance(Change::Add(amount))
    }

    /// Remove a member's withdrawal from the lane's balance.
    pub(crate) fn withdraw(self, amount: Amount) -> Self {
        self.change_balance(Change::Remove(amount))
    }

    /// The remainder belongs to the old balance, so it goes unallocated.
    fn change_balance(mut self, change: Change) -> Self {
        let balance = change.apply(self.balance);

        if balance != self.balance {
            self.balance = balance;
            self.remainder = N::ZERO;
        }

        self
    }
}

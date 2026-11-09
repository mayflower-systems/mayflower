use mayf_fixed_number::FixedNumber;

use crate::numbers::Change;
use crate::{Amount, Error, Result};

/// A lane's pool: its total balance, and the credits paid per unit of balance
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanePool<N> {
    /// The sum of every member's balance in the lane
    pub balance: Amount,
    /// Credits paid per unit of balance since the lane began
    pub reward_index: N,
    /// Credits received that were too small to add to the index at this balance
    pub remainder: N,
    pub unallocated: N,
}

impl<N: FixedNumber> Default for LanePool<N> {
    fn default() -> Self {
        Self {
            balance: 0,
            reward_index: N::ZERO,
            remainder: N::ZERO,
            unallocated: N::ZERO,
        }
    }
}

impl<N: FixedNumber> LanePool<N> {
    /// Pay `allocation` across the pool's balance. While no one holds the lane, it goes
    /// unallocated.
    pub(crate) fn absorb(mut self, allocation: N) -> Result<Self> {
        if allocation < N::ZERO {
            return Err(Error::InvalidEmission);
        }
        let available = self.remainder.checked_add(allocation)?;
        if self.balance == 0 {
            self.unallocated = self.unallocated.checked_add(available)?;
            self.remainder = N::ZERO;
        } else {
            let balance = N::checked_from_u64(self.balance)?;
            let increment = available.checked_div_floor(balance)?;
            // Multiplication by the integer balance preserves the exact remainder.
            let allocated = increment.checked_mul_floor(balance)?;
            self.reward_index = self.reward_index.checked_add(increment)?;
            self.remainder = available.checked_sub(allocated)?;
        }
        Ok(self)
    }

    /// Close shared source dust before any recipient eligibility changes.
    pub(crate) fn close_eligibility(&mut self) -> Result<()> {
        self.unallocated = self.unallocated.checked_add(self.remainder)?;
        self.remainder = N::ZERO;
        Ok(())
    }

    /// Add a member's deposit to the lane's balance.
    pub(crate) fn deposit(self, amount: Amount) -> Result<Self> {
        self.change_balance(Change::Add(amount))
    }

    /// Remove a member's withdrawal from the lane's balance.
    pub(crate) fn withdraw(self, amount: Amount) -> Result<Self> {
        self.change_balance(Change::Remove(amount))
    }

    /// The remainder belongs to the old balance, so it goes unallocated.
    fn change_balance(mut self, change: Change) -> Result<Self> {
        let balance = change.apply(self.balance)?;

        if balance != self.balance {
            self.close_eligibility()?;
        }
        self.balance = balance;

        Ok(self)
    }
}

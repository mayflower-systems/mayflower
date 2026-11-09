//! The kernel's whole numbers: token amounts and seconds.
//!
//! Fractional values, such as indexes and kept rewards, use the host's fixed-point type `N`
//! through [`FixedNumber`](mayf_fixed_number::FixedNumber).

/// Whole token base units: credits, asset balances, gauge votes, and join fees
pub type Amount = u64;

/// A Unix time or a duration, in whole seconds
pub type Seconds = u64;

/// A change to one balance, and so to every total that includes it
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Add(Amount),
    Remove(Amount),
}

impl Change {
    /// The change from `old` to `new`
    pub fn between(old: Amount, new: Amount) -> Self {
        if new >= old {
            Self::Add(new - old)
        } else {
            Self::Remove(old - new)
        }
    }

    /// Apply the change to `total`, which includes the changed balance.
    pub fn apply(self, total: Amount) -> crate::Result<Amount> {
        match self {
            Self::Add(amount) => total.checked_add(amount),
            Self::Remove(amount) => total.checked_sub(amount),
        }
        .ok_or(crate::Error::NumberOutOfRange)
    }
}

use core::num::NonZero;

use crate::{Amount, Seconds};

/// A rate of emission: `amount` credit base units every `period` seconds
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmissionRate {
    pub amount: Amount,
    pub period: NonZero<Seconds>,
}

impl EmissionRate {
    /// The credits emitted over `elapsed` seconds, rounded down, up to `Amount::MAX`
    pub(crate) fn over(self, elapsed: Seconds) -> Amount {
        let emitted = u128::from(self.amount) * u128::from(elapsed) / u128::from(self.period.get());
        Amount::try_from(emitted).unwrap_or(Amount::MAX)
    }
}

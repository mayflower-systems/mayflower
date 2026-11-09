//! Authenticated cumulative emission supplied by the host's emitter.

use mayf_fixed_number::FixedNumber;

use crate::{Error, Result, Seconds};

/// Integrated scheduled emission, not outstanding token supply.
/// The host authenticates the emitter and its declining rate independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Emission<N> {
    pub at: Seconds,
    pub total: N,
}

impl<N: FixedNumber> Default for Emission<N> {
    fn default() -> Self {
        Self {
            at: 0,
            total: N::ZERO,
        }
    }
}

impl<N: FixedNumber> Emission<N> {
    pub(crate) fn validate_after(self, previous: Self) -> Result<()> {
        if self.at < previous.at {
            return Err(Error::TimeWentBack);
        }
        if previous.total < N::ZERO
            || self.total < previous.total
            || (self.at == previous.at && self.total != previous.total)
        {
            return Err(Error::InvalidEmission);
        }
        Ok(())
    }
}

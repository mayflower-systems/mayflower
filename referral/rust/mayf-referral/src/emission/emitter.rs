use super::EmissionRate;
use crate::{Amount, Seconds};

/// Emits credits at a rate the host can change
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Emitter {
    pub rate: EmissionRate,
    /// When the current rate began
    pub since: Seconds,
    /// The total emitted before `since`
    pub emitted: Amount,
}

impl Emitter {
    pub(crate) fn new(rate: EmissionRate, now: Seconds) -> Self {
        Self {
            rate,
            since: now,
            emitted: 0,
        }
    }

    /// The total emitted up to `now`. It stops at `Amount::MAX`, the largest token supply.
    pub(crate) fn emitted_at(&self, now: Seconds) -> Amount {
        let elapsed = now.saturating_sub(self.since);
        self.emitted.saturating_add(self.rate.over(elapsed))
    }

    /// Emit at `rate` from `now` on.
    pub(crate) fn with_rate(mut self, rate: EmissionRate, now: Seconds) -> Self {
        self.emitted = self.emitted_at(now);
        self.since = self.since.max(now);
        self.rate = rate;
        self
    }
}

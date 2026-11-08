use mayf_fixed_number::{FixedNumber, MathResult};

use crate::Seconds;
use crate::emission::Emitter;
use crate::tree::{NetworkTerms, Offer};
use crate::weighting::Gauge;

/// The global record: the network's terms, the price of a top-level seat, the emitter, and the
/// gauge. The host stores one per network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Protocol<N> {
    pub terms: NetworkTerms,
    /// The price for a new member to take a top-level seat, under no parent
    pub top_level_offer: Offer,
    pub emitter: Emitter,
    pub gauge: Gauge<N>,
}

impl<N: FixedNumber> Protocol<N> {
    /// The gauge with all emission up to `now` split among the lanes
    pub(crate) fn gauge_at(&self, now: Seconds) -> MathResult<Gauge<N>> {
        self.gauge.settled(self.emitter.emitted_at(now))
    }
}

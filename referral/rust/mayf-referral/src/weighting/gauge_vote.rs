use crate::Amount;

/// One voter's gauge votes on one lane.
/// A vote of zero is the same as no vote, so the host may delete it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GaugeVote {
    pub votes: Amount,
}

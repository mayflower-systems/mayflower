//! Weighting: split emission among lanes by gauge votes.
//!
//! A lane's weight is the sum of its gauge votes. The gauge keeps one index: the credits emitted
//! per unit of weight. A lane stores its weight and the index it last saw, and its allocation is
//! its weight times the index's growth. So a change to one lane's votes settles only that lane.
//! While no lane has votes, nothing is emitted.

mod gauge;
mod gauge_vote;
mod lane_gauge;

pub use gauge::Gauge;
pub use gauge_vote::GaugeVote;
pub use lane_gauge::LaneGauge;

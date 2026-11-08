//! Emission: credits emitted at a known rate.
//!
//! The emitter stores the total emitted before its current rate began. Anyone computes the
//! total at `now` from that anchor, so the emitter is written only when its rate changes.

mod emitter;
mod rate;

pub use emitter::Emitter;
pub use rate::EmissionRate;

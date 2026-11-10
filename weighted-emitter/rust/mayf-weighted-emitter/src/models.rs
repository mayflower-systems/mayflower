mod emitter;
mod gauge;

pub use emitter::Emitter;
pub use gauge::Gauge;

/// Whole token base units, or whole votes backed by the host.
pub type Amount = u64;
pub type Seconds = u64;

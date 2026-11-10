#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod actions;
mod error;
pub mod models;
mod schedule;

pub use actions::*;
pub use error::{Result, WeightedEmitterError};
pub use models::{Amount, Emitter, Gauge, Seconds};
pub use schedule::{SECONDS_PER_YEAR, Schedule};

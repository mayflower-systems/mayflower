#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod calculator;
mod error;
mod segmentation;
mod shifted;

pub use calculator::{SegmentedScheduleCalculator, compile_segmented_schedule};
pub use error::{ScheduleError, ScheduleResult};
pub use segmentation::{MainReference, Segmentation};
pub use shifted::ShiftedScheduleCalculator;

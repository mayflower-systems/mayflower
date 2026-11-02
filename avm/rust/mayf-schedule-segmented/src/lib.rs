#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod calculator;
pub mod geometry;
mod schedule;

pub use geometry::{Segmentation, SegmentationError, SegmentedRaiseFloorProposal};
pub use schedule::{
    BuyQuote, FloorRaise, FloorSchedule, RaisableFloorSchedule, Schedule, ScheduleError,
    ScheduleResult, SegmentedSchedule, SellQuote,
};

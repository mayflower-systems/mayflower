#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod calculator;
mod engine;
mod error;
mod state;

pub use calculator::{ScheduleCalculator, compile_schedule};
pub use engine::{
    AvmCalculator, CharacteristicEngine, FloorRaisePolicy, FloorRaiseProposal, FloorRaiseQuote,
    TradeQuote,
};
pub use error::{Error, Result};
pub use state::{Accounting, MarketState, Segmentation};

pub use mayf_characteristic::{AffineAnchor, AffineTransform, Characteristic};
pub use mayf_monotone::{MonotoneCalculator, MonotoneCalculatorError, MonotoneCalculatorResult};

#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod calculator;
mod characteristic;
mod error;
mod market;
pub mod solver;
mod state;

pub use calculator::ScheduleCalculator;
pub use characteristic::{
    CharacteristicEngine, CharacteristicEngineError, CharacteristicEngineResult,
    LocalCharacteristic, SpanRequest, SpanResponse,
};
pub use error::{Error, Result};
pub use market::{
    AvmCalculator, FloorRaisePolicy, FloorRaiseProposal, FloorRaiseQuote, TradeQuote,
};
pub use state::{Accounting, MarketState, Segmentation};

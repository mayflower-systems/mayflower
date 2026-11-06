#![forbid(unsafe_code)]

//! Pure reward-clearing auction mathematics and accounting transitions.
//!
//! Actions take numeric state and return replacement state with exact semantic
//! amounts. The host supplies the correct auction, position, current newest lot,
//! and time, then applies each receipt atomically. Authentication, persistence,
//! custody, fees, quotes, staging, and account lifetime belong to the host.
//!
//! Principal and rewards are whole `u64` accounting units. Shares, weights, and
//! fractions use the caller's fixed-number driver with explicit rounding.
//! Claims floor each auction's entitlement independently.

mod actions;
mod amount;
mod error;
mod math;
pub mod models;
mod state;
mod validation;

pub use actions::*;
pub use amount::Amount;
pub use error::{Result, RewardClearingError};
pub use state::{
    EpochCounter, EpochDuration, EpochSchedule, EpochSnapshot, EpochTerms, EpochWindow,
    ExitPenaltyExponent, FinalizedEpoch, FinalizedEpochSnapshot, Lot, LotSnapshot, Position,
    PositionSnapshot, RampBonusSlope, Seconds,
};

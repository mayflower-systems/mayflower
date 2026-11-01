#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]

mod actions;
mod error;
mod numeric;
mod state;

pub use actions::*;
pub use error::{ReferralError, Result};
pub use numeric::{ReferralPoints, ReferralRate, ReferralTreeConfig, ReferralTreeNode};
pub use referral_tree;
pub use state::*;

#[cfg(test)]
mod tests;

#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod conversions;
mod kernels;
mod scaled;
mod signed;
mod unsigned;
mod wide_unsigned;

#[cfg(feature = "reference")]
pub mod reference;

pub use scaled::{I64Scaled, U256Scaled};
pub use signed::I64F64;
pub use unsigned::{U64F64, U128F128};
pub use wide_unsigned::U192F64;

#![no_std]
#![doc = include_str!("../README.md")]

//! Fixed-point numeric traits and branded numeric wrappers.
//!
//! This crate owns the shared numeric foundation for Mayflower crates.

mod bounded;
mod bounds;
mod capabilities;
pub mod certified;
pub mod driver;
mod fixed;
mod refined;

pub use bounded::Bounded;
pub use bounds::Bounds;
pub use capabilities::{SignedFixed, UnsignedFixed};

pub use fixed::{
    BoundedExponential, DirectedRounding, FixedNumber, IntegralRounding, MathError, MathResult,
};
pub use refined::{AboveOne, NonNeg, Pos};

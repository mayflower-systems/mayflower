#![no_std]
#![forbid(unsafe_code)]

//! Pure AVM actions that return next-state receipts for the caller to settle.

pub mod actions;
pub mod fees;
pub mod state;

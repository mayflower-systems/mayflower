//! Log sink program ID, shared by the CPI helper and the reader.
//!
//! [`ID_STR`] is the base58 address. [`ID`] is decoded at compile time.

#![no_std]

/// The log sink program address in base58.
pub const ID_STR: &str = "LogtC4Vb4zbGYXY3SY7Jg8ge4MU8K8oznv9sLuMEcym";

solana_address::declare_id!(ID_STR);

//! Optional allocating arithmetic for host applications and independent tests.
//!
//! `DashuFixed12` implements the same fixed-point contracts as the stack-only
//! drivers. The arbitrary-precision integers let tests derive exact reference
//! results without sharing those drivers' arithmetic kernels. They are integer
//! building blocks, not `FixedNumber` implementations: unbounded integers do
//! not satisfy that trait's `Copy` contract.

pub use dashu::integer::{IBig, UBig};
pub use mayf_fixed_dashu::DashuFixed12;

#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod linear;

pub use linear::LinearCharacteristic;
pub use mayf_characteristic_linear::{LinearParams, LinearParamsError};

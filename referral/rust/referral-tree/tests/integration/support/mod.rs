//! Shared grid choices for protocol examples; implementations live in drivers.
#![allow(dead_code)]
use mayf_fixed_drivers::{I64Scaled, U256Scaled};

pub type Tenths = I64Scaled<10>;
pub type WideTenths = U256Scaled<10>;
pub type WidePico = U256Scaled<1_000_000_000_000>;

pub const fn tenths(raw: i64) -> WideTenths {
    assert!(raw >= 0, "tenths are nonnegative");
    WideTenths::from_scaled_u128(raw as u128)
}

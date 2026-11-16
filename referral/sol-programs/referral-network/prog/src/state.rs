//! Persisted host records. Every `*_q192f64` field is an unsigned Q192.64
//! fixed-point value encoded as 32 little-endian bytes, matching `U192F64`.
//! Carry fields retain the kernel's numerator units; they are not token amounts.

pub(crate) mod com;
pub(crate) mod lane;
pub(crate) mod lane_node;
pub(crate) mod network;
pub(crate) mod node;

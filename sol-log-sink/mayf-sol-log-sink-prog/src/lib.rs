//! A Solana log sink for event data.
//!
//! Call with raw event bytes as instruction data and no accounts or signers.
//! The entrypoint ignores the input buffer and always succeeds. Read payloads
//! from the transaction's recorded inner instructions.
//!
//! Any program can call the sink. Readers must check transaction success and
//! identify the immediate caller from the trace.

/// Accept every call without reading the loader's input buffer.
#[cfg(target_os = "solana")]
#[unsafe(no_mangle)]
pub extern "C" fn entrypoint(_input: *mut u8) -> u64 {
    0
}

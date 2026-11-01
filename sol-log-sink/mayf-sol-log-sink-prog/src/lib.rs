//! A Solana log sink that accepts only cross-program invocations (CPIs).
//!
//! Call with raw event bytes as instruction data and no accounts or signers.
//! The entrypoint checks stack height and ignores the input buffer. Read
//! payloads from the transaction's recorded inner instructions.
//!
//! Any program can call the sink. Readers must check transaction success and
//! identify the immediate caller from the trace.

/// Shared program ID and address helpers.
pub use mayf_sol_log_sink_id::{ID, ID_STR, check_id, id};

/// Stack height of a top-level transaction instruction.
const TRANSACTION_LEVEL_STACK_HEIGHT: u64 = 1;

/// Successful program return status.
pub const SUCCESS: u64 = 0;

/// Custom error code for a direct transaction invocation.
pub const ERROR_DIRECT_INVOCATION: u64 = 1;

/// Return [`SUCCESS`] for stack heights above 1, or
/// [`ERROR_DIRECT_INVOCATION`] otherwise.
pub const fn invocation_status(stack_height: u64) -> u64 {
    if stack_height > TRANSACTION_LEVEL_STACK_HEIGHT {
        SUCCESS
    } else {
        ERROR_DIRECT_INVOCATION
    }
}

/// Check CPI stack height without reading the loader's input buffer.
#[cfg(target_os = "solana")]
#[unsafe(no_mangle)]
pub extern "C" fn entrypoint(_input: *mut u8) -> u64 {
    let stack_height = unsafe { solana_define_syscall::definitions::sol_get_stack_height() };
    let status = invocation_status(stack_height);
    if status != SUCCESS {
        explain();
    }
    status
}

// Log the reason for rejecting a direct invocation.
#[cfg(target_os = "solana")]
fn explain() {
    let message = "log sink: only another program may invoke this";
    unsafe {
        solana_define_syscall::definitions::sol_log_(message.as_ptr(), message.len() as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_inner_instruction_is_accepted() {
        for stack_height in [0, TRANSACTION_LEVEL_STACK_HEIGHT] {
            assert_eq!(invocation_status(stack_height), ERROR_DIRECT_INVOCATION);
        }
        for stack_height in [2, 3, 4] {
            assert_eq!(invocation_status(stack_height), SUCCESS);
        }
    }
}

//! Send event bytes to the log sink through CPI.
//!
//! [`log`] borrows its payload and passes no accounts or signer seeds.
//! It uses the SDK's safe CPI helper without allocating or copying payload bytes.

#![forbid(unsafe_code)]

use solana_account_view::AccountView;
use solana_instruction_view::{InstructionView, cpi::invoke};
use solana_program_error::ProgramResult;

/// The log sink program ID in binary and base58 form.
pub use mayf_sol_log_sink_id::{ID, ID_STR};

/// Invoke the log sink with `payload` as the entire instruction data.
///
/// The caller's instruction must include [`ID`] in its account list.
/// The CPI passes no accounts or signer seeds and adds no discriminator.
/// This helper borrows the payload without heap allocation or copying.
///
/// On host builds, returns `Ok(())` without recording an event.
///
/// ```no_run
/// # fn main() -> Result<(), solana_program_error::ProgramError> {
/// mayf_sol_log_sink_cpi::log(b"event bytes")?;
/// # Ok(())
/// # }
/// ```
pub fn log(payload: &[u8]) -> ProgramResult {
    let instruction = InstructionView {
        program_id: &ID,
        accounts: &[],
        data: payload,
    };
    // Zero accounts; `invoke` supplies no signer seeds.
    invoke::<0, AccountView>(&instruction, &[])
}

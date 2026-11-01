# `mayf-sol-log-sink-cpi`

Invoke the log sink with a borrowed byte slice.

```rust
use solana_program_error::ProgramResult;

fn emit_event(payload: &[u8]) -> ProgramResult {
    mayf_sol_log_sink_cpi::log(payload)
}
```

`log` passes the bytes unchanged, with no discriminator, accounts, or signer
seeds. It uses the SDK's safe
[`invoke`](https://docs.rs/solana-instruction-view/latest/solana_instruction_view/cpi/fn.invoke.html)
with an `InstructionView` and a zero-length account array. The helper allocates
no heap memory and copies no payload bytes. This crate forbids unsafe code.

On the host, `log` returns `Ok(())` without recording an event.

## Account setup

The caller's instruction must include the sink program as a read-only,
non-signer account. The CPI passes zero accounts to the sink. Forward the sink
program account through any intermediate calls that need it.

`ID` and `ID_STR` come from the shared
[`mayf-sol-log-sink-id`](../mayf-sol-log-sink-id/README.md) crate.
For Anchor, add the sink to the calling instruction's account context:

```rust
use anchor_lang::prelude::*;

pub const LOG_SINK_PROGRAM_ID: Pubkey =
    Pubkey::new_from_array(mayf_sol_log_sink_cpi::ID.to_bytes());

#[derive(Accounts)]
pub struct EmitEvent<'info> {
    /// CHECK: The address constraint identifies the log sink program.
    #[account(address = LOG_SINK_PROGRAM_ID)]
    pub log_sink_program: UncheckedAccount<'info>,
}
```

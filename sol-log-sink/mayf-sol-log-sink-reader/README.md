# `mayf-sol-log-sink-reader`

Extract the payloads a specific program sent to the log sink.

```rust
use mayf_sol_log_sink_reader::{TraceError, logged_by};
use solana_address::Address;
use solana_message::{Message, inner_instruction::InnerInstructionsList};

fn events<'a>(
    message: &Message,
    trace: &'a InnerInstructionsList,
    program: &Address,
) -> Result<Vec<&'a [u8]>, TraceError> {
    logged_by(message, trace, program)
}
```

The result contains borrowed payload slices in execution order. Only calls
made directly by the requested program are included. Nested calls are matched
using the active invocation stack, not the preceding instruction alone.

## Input requirements

- A legacy `Message`; versioned messages are not supported.
- One trace group per top-level instruction, in order. Use an empty group when
  that instruction made no CPIs.
- Decoded instruction data and recorded stack heights. Raw RPC JSON must be
  converted to these types first.
- A successful transaction. The function does not check transaction status.

Invalid group counts, program indexes, or stack transitions return
`TraceError`; no partial result is returned.

## Test

From the repository root:

```bash
cargo test -p mayf-sol-log-sink-reader --locked
```

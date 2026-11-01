# `mayf-sol-log-sink-prog`

Solana program that accepts event data through CPI.

The program ID comes from [`mayf-sol-log-sink-id`](../mayf-sol-log-sink-id/README.md).

## Instruction

| Field | Value |
| --- | --- |
| Data | Raw event bytes; empty data is allowed. |
| Accounts | None required or read. |
| Signers | None required. |
| Discriminator | None. |

The program succeeds when the invocation stack height exceeds 1. Otherwise,
it logs an error and returns `ProgramError::Custom(1)`.

The entrypoint calls `sol_get_stack_height` without deserializing the loader's
input buffer. It does not inspect the payload or write state. Any program can
invoke it; readers must identify the caller from the transaction trace.

## Build

Building the program needs `cargo build-sbf`, which comes with the Solana
(Agave) CLI tools. From the repository root:

```bash
cargo build-sbf --manifest-path sol-log-sink/mayf-sol-log-sink-prog/Cargo.toml
cargo test -p mayf-sol-log-sink-prog --all-targets --locked
```

Output: `target/deploy/mayf_sol_log_sink_prog.so`.

## Deployment

The repository includes no deployment keypair. To deploy a new instance, set
`ID_STR` in [mayf-sol-log-sink-id/src/lib.rs](../mayf-sol-log-sink-id/src/lib.rs)
to your program keypair's public key, then rebuild the program, callers, and
readers.

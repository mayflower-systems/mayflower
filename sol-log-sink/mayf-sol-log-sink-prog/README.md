# `mayf-sol-log-sink-prog`

Solana program that accepts event data.

The program does not know its own address. Callers and readers get it from
[`mayf-sol-log-sink-id`](../mayf-sol-log-sink-id/README.md).

## Instruction

| Field         | Value                                   |
| ------------- | --------------------------------------- |
| Data          | Raw event bytes; empty data is allowed. |
| Accounts      | None required or read.                  |
| Signers       | None required.                          |
| Discriminator | None.                                   |

The program always succeeds. The entrypoint returns without reading the
loader's input buffer. It does not inspect the payload or write state.

Any program can invoke it, and a transaction can also call it directly.
Readers must identify the caller from the transaction trace.
[`mayf-sol-log-sink-reader`](../mayf-sol-log-sink-reader/README.md) returns
only payloads sent through CPI by a given program.

## Build

Building the program needs `cargo build-sbf`, which comes with the Solana
(Agave) CLI tools. Run the build script:

```bash
./build_program.sh
```

The script works from any directory.

Output: `out/mayf_sol_log_sink_prog.so`.

## Deployment

The repository includes no deployment keypair. To deploy a new instance, set
`ID_STR` in [mayf-sol-log-sink-id/src/lib.rs](../mayf-sol-log-sink-id/src/lib.rs)
to your program keypair's public key, then rebuild the callers and readers.
The program binary does not change.

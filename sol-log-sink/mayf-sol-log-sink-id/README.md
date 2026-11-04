# `mayf-sol-log-sink-id`

Log sink program ID, shared by the CPI helper and the reader.

`ID_STR` holds the base58 address. `ID` holds the address decoded at compile
time. The crate also exports the standard `id()` and `check_id()` helpers.

This crate uses `no_std`, forbids unsafe code, and depends only on
`solana-address` with its `decode` feature. Using the constants requires no
runtime parsing or allocation.

To target a different deployment, change `ID_STR` in [src/lib.rs](src/lib.rs),
then rebuild the callers and readers. The program itself does not use this
crate.

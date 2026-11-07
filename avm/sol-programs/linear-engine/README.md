# Linear characteristic engine

Run `./build.sh` to build all artifacts. The script resolves paths from its own
location, so it also works when invoked from the repository root or elsewhere.
It requires `cargo-build-sbf` and uses Solana platform tools `v1.58` plus the
repository's pinned Rust toolchain and dependencies.

| Directory | Contents |
| --- | --- |
| `prog/` | Anchor program source and native arithmetic tests |
| `bin/` | Built `mayf_avm_engine_linear_sol.so` |
| `idl/` | Generated `mayf_avm_engine_linear_sol.json` |
| `cpi/` | Generated Rust library `mayf-avm-engine-linear-sol-cpi` |

The script builds the program with its default features, extracts its IDL,
generates `cpi/src/` using `mayf-anchor-idl-to-gen`, formats the generated code,
and checks that the CPI crate compiles. Cargo's intermediate files and automatic
deployment keypair remain in `target/`.

Keep `cpi/Cargo.toml` as the crate definition; regenerate `cpi/src/` instead of
editing it. Generated bindings take the program address through
`MayfAvmEngineLinearApi::new(program_id)` and include instruction builders,
account layouts, response models, and error codes. The caller invokes the built
instruction and decodes return data using these models.

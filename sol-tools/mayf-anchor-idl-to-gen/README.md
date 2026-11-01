# `mayf-anchor-idl-to-gen`

Generate Rust and TypeScript clients for an Anchor program. The generated
clients do not depend on Anchor.

The crate has two parts. The library converts an IDL to client source in
memory; it reads no files and starts no processes. The `anchor-idl-to-gen`
binary owns every side effect: it reads the IDL, writes the files, and runs
`cargo` to build an IDL.

## Command line

From this workspace, run the binary with
`cargo run -p mayf-anchor-idl-to-gen -- <command>`.

Build an IDL from an Anchor program crate:

```bash
anchor-idl-to-gen build-idl programs/vault --output idl/vault.json
```

`build-idl` uses `anchor-lang-idl`, the library behind `anchor idl build`. It
needs neither the Anchor CLI nor the Solana CLI. It runs
`cargo test --features idl-build --locked` in the program directory, so the
program crate needs Anchor's usual `idl-build` feature. It ignores
`RUSTUP_TOOLCHAIN`, so rustup picks the toolchain from the program's
`rust-toolchain.toml`.

Generate Rust or TypeScript bindings from that IDL:

```bash
anchor-idl-to-gen generate-rust idl/vault.json \
  --program-name Vault --output-dir clients/vault-rs/src

anchor-idl-to-gen generate-typescript idl/vault.json \
  --program-name Vault --output-dir clients/vault-ts/src
```

`--program-name` is the UpperCamelCase prefix for program-specific names, such
as `VaultApi` and `VaultError`. The generator does not format its output. Run
the target project's formatter, such as `cargo fmt`, after each generation.

## Library

```rust
use mayf_anchor_idl_to_gen::{GenerateConfig, materialization, parse_anchor_idl};

let program = parse_anchor_idl(GenerateConfig {
    idl_json,
    program_name: "Vault".to_owned(),
})?;
let rust_files = materialization::rust::materialize(&program)?;
let typescript_files = materialization::typescript::materialize(&program)?;
```

`parse_anchor_idl` returns `ProgramData`, a model with no Anchor or
target-language types. Each materializer returns a file bundle; its `files()`
method lists each file name with its contents. `generate` and
`generate_typescript` run both steps in one call.

## Generated clients

Generated clients never contain a program address. Callers supply the
deployment's address to the instruction and PDA helpers.

### Rust

`generate-rust` writes `lib.rs`, `accounts.rs`, `errors.rs`, `events.rs`,
`instructions.rs`, `models.rs`, and `pda.rs`. The code depends on `borsh`,
`solana-address`, and `solana-instruction`. Create one `<Program>Api` value
for the target deployment, then call the generated PDA and instruction traits
through it.

`instructions.rs` also defines a `<Program>Instruction` enum with `ALL`,
`name()`, `discriminator()`, and `from_discriminator()`. Use it to dispatch on
every instruction without keeping a second list. Generation fails if two
instructions share a discriminator.

### TypeScript

`generate-typescript` writes `index.ts`, `accounts.ts`, `codecs.ts`,
`errors.ts`, `events.ts`, `instructions.ts`, `models.ts`, and `pda.ts`. Codecs
are plain `borsh` 2.x schemas, with no generated classes or custom readers.
Only the Solana boundary uses `bs58` and `@solana/web3.js` v1. Models hold
bytes as `ReadonlyArray<number>` and public keys as `PublicKeyBytes`;
`codecs.ts` converts these to and from web3.js `PublicKey` values.

Each instruction gets two builders. For an instruction named `deposit`:

- `createDepositAccountMetas` returns the ordered `AccountMeta[]`. Use it to
  forward the accounts as another instruction's remaining accounts.
- `createDepositInstruction` adds the encoded instruction data to that list.

The full builder calls the account-meta builder, so the two cannot disagree on
order or flags. When a PDA depends on an instruction argument, the account-meta
builder takes a small `DepositAccountMetaSeeds` object with only those values.
It needs a program ID only to derive a PDA or to fill an omitted optional
account, for which Anchor expects the program's own address.

## PDAs

The generator derives a PDA when each of its non-constant seeds is one of:

- the address of another account in the instruction;
- an instruction argument of type `u32`, encoded little-endian;
- an instruction argument that is a fixed byte array of 1 to 32 bytes, used
  as-is.

An argument seed may name a field inside a struct argument, such as `args.id`.
Generation fails for any other seed, such as a field of an account's data. For
those programs, pass `--manual-pdas`: the clients then take every PDA account
as a caller-supplied address, and the existing `pda.rs` or `pda.ts` is left
unchanged.

## Tests

Unit tests build their input in Rust and read no files.
[`src/test_support.rs`](src/test_support.rs) holds one small, generic Anchor
IDL that covers the usual range of IDL values, plus a few programs that each
isolate one generator feature.

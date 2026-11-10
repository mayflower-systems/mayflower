#!/usr/bin/env bash
set -euo pipefail

program_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$program_dir/../../.." && pwd)"
program_name="mayf_weighted_emitter_sol"
build_dir="$repo_root/target/sol-programs/$program_name"
idl_path="$program_dir/idl/$program_name.json"

cd -- "$repo_root"
mkdir -p "$program_dir/bin" "$program_dir/idl"

# Keep cargo-build-sbf's automatically generated keypair in target/.
cargo build-sbf \
    --manifest-path "$program_dir/prog/Cargo.toml" \
    --tools-version v1.58 \
    --sbf-out-dir "$build_dir" \
    -- --locked

cargo run --locked -p mayf-anchor-idl-to-gen -- \
    build-idl "$program_dir/prog" --output "$idl_path"

# Both clients take explicit account addresses, including PDAs supplied by callers.
cargo run --locked -p mayf-anchor-idl-to-gen -- \
    generate-rust "$idl_path" \
    --program-name MayfWeightedEmitter \
    --manual-pdas \
    --output-dir "$program_dir/cpi/src"

cargo fmt -p mayf-weighted-emitter-sol-cpi
cargo check --locked -p mayf-weighted-emitter-sol-cpi

cargo run --locked -p mayf-anchor-idl-to-gen -- \
    generate-typescript "$idl_path" \
    --program-name MayfWeightedEmitter \
    --output-dir "$program_dir/ts/src"

pnpm exec prettier --write --log-level=warn "$program_dir/ts"
pnpm --dir "$program_dir/ts" run build

install -m 644 "$build_dir/$program_name.so" "$program_dir/bin/$program_name.so"

# Exercise a real caller PDA in LiteSVM, including the runtime signer rules.
caller_name="mayf_weighted_emitter_test_caller"
caller_build_dir="$repo_root/target/sol-programs/$caller_name"
cargo build-sbf \
    --manifest-path "$program_dir/litesvm/test-program/Cargo.toml" \
    --tools-version v1.58 \
    --sbf-out-dir "$caller_build_dir" \
    -- --locked
mkdir -p "$program_dir/litesvm/bin"
install -m 644 "$caller_build_dir/$caller_name.so" "$program_dir/litesvm/bin/$caller_name.so"

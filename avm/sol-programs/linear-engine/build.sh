#!/usr/bin/env bash
set -euo pipefail

program_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$program_dir/../../.." && pwd)"
program_name="mayf_avm_engine_linear_sol"
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

cargo run --locked -p mayf-anchor-idl-to-gen -- \
    generate-rust "$idl_path" \
    --program-name MayfAvmEngineLinear \
    --output-dir "$program_dir/cpi/src"

cargo fmt -p mayf-avm-engine-linear-sol-cpi
cargo check --locked -p mayf-avm-engine-linear-sol-cpi

cargo run --locked -p mayf-anchor-idl-to-gen -- \
    generate-typescript "$idl_path" \
    --program-name MayfAvmEngineLinear \
    --output-dir "$program_dir/ts/src"

pnpm exec prettier --write --log-level=warn "$program_dir/ts"
pnpm --dir "$program_dir/ts" run build

install -m 644 "$build_dir/$program_name.so" "$program_dir/bin/$program_name.so"

#!/usr/bin/env bash
# Builds the mayf-sol-log-sink-prog Solana program and copies the .so file
# into ./out next to this script.
#
# Needs `cargo build-sbf` from the Solana (Agave) CLI tools.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out_dir="${script_dir}/out"

mkdir -p "${out_dir}"

cargo build-sbf \
  --manifest-path "${script_dir}/Cargo.toml" \
  --sbf-out-dir "${out_dir}" \
  -- --locked

# cargo build-sbf writes a random program keypair next to the .so file when
# none exists. It has no flag to skip this. Delete it so no key lands in out/.
rm -f "${out_dir}/mayf_sol_log_sink_prog-keypair.json"

echo "Built ${out_dir}/mayf_sol_log_sink_prog.so"

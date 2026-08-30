#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo build --lib --quiet
rustc --edition=2021 fuzz/fuzz_target.rs --extern finlang_core="$(find target/debug/deps -name 'libfinlang_core-*.rlib' | head -1)" -L dependency=target/debug/deps -o target/fuzz_target
iterations="${1:-10000}"
for ((i=0; i<iterations; i++)); do
  seed=$((i * 1103515245 + 12345))
  printf '%s\n' "$seed" | target/fuzz_target >/dev/null
done
echo "fuzz iterations: $iterations"

#!/usr/bin/env bash
# Fast path/spec coverage: use an existing binary when possible.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
if [[ -x target/release/specsync ]]; then
  exec target/release/specsync check --strict --require-coverage 100 --force
fi
if [[ -x target/debug/specsync ]]; then
  exec target/debug/specsync check --strict --require-coverage 100 --force
fi
exec cargo run --quiet --release -- check --strict --require-coverage 100 --force

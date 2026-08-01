#!/usr/bin/env bash
# Mandatory local gate before `git push` on CorvidLabs/spec-sync.
# Mirrors the CI jobs that commonly fail when skipped: fmt + clippy + types + spec-check.
#
# Preferred:
#   fledge lanes run pre-push
# Equivalent:
#   ./scripts/pre-push-gate.sh
#
# Exit 0 only when every step succeeds. Do not push on non-zero.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> cargo fmt --check"
cargo fmt -- --check

echo "==> cargo clippy -- -D warnings"
cargo clippy -- -D warnings

echo "==> cargo check"
cargo check

echo "==> cargo run -- check --strict --require-coverage 100 --force"
cargo run --quiet -- check --strict --require-coverage 100 --force

echo "==> pre-push gate PASS — safe to git push"

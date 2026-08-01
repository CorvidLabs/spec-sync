#!/usr/bin/env bash
# Fast mandatory gate before `git push` on CorvidLabs/spec-sync.
#
# Goal: catch the failures we keep shipping to CI (fmt + path/export coverage)
# in ~seconds to ~1–2 minutes on a warm machine — NOT a full test suite.
#
# Preferred:
#   fledge lanes run pre-push
# Equivalent:
#   ./scripts/pre-push-gate.sh
#
# Full trust (slower, before merge-ready):
#   fledge lanes run verify
#
# Exit 0 only when every step succeeds. Do not push on non-zero.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ts() { date +%s; }
elapsed() {
  local start="$1" end
  end=$(ts)
  echo $((end - start))
}

total_start=$(ts)
echo "==> pre-push gate (fast) — started $(date -u +%H:%M:%S)Z"

# 1) Format — seconds
step_start=$(ts)
echo "==> [1/3] cargo fmt --check"
cargo fmt -- --check
echo "    ok ($(( $(ts) - step_start ))s)"

# 2) Types only — incremental; skip clippy here (clipply lives in verify/ci)
step_start=$(ts)
echo "==> [2/3] cargo check"
cargo check --quiet
echo "    ok ($(( $(ts) - step_start ))s)"

# 3) Spec/path coverage — prefer release binary (no recompile)
step_start=$(ts)
echo "==> [3/3] strict path/spec coverage"
if [[ -x target/release/specsync ]]; then
  SPECSYNC_BIN=target/release/specsync
  echo "    using $SPECSYNC_BIN"
elif [[ -x target/debug/specsync ]]; then
  SPECSYNC_BIN=target/debug/specsync
  echo "    using $SPECSYNC_BIN (debug)"
else
  echo "    building release binary once (cached thereafter)…"
  cargo build --release --quiet
  SPECSYNC_BIN=target/release/specsync
fi
"$SPECSYNC_BIN" check --strict --require-coverage 100 --force
echo "    ok ($(( $(ts) - step_start ))s)"

total=$(( $(ts) - total_start ))
echo "==> pre-push gate PASS in ${total}s — safe to git push"
echo "    (for full test+clippy+release: fledge lanes run verify)"

#!/usr/bin/env bash
# Fast mandatory gate before `git push` (~seconds–2 min warm).
# Not full cargo test / clippy — those stay in verify/CI.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ts() { date +%s; }

total_start=$(ts)
echo "==> pre-push gate (fast) — started $(date -u +%H:%M:%S)Z"

# Archive-only tip: unpushed/diff files are solely under .specsync/archive/
# (and optional change-sequence). Skip product path coverage and change audit —
# after finalize there is no active delivering change, and CI archive-integrity
# owns that path.
archive_tip=false
if git rev-parse --abbrev-ref --symbolic-full-name '@{u}' >/dev/null 2>&1; then
  files=$(
    {
      git diff --name-only --relative '@{u}'...HEAD 2>/dev/null || true
      git status --porcelain -uall | awk '{print $NF}'
    } | sed '/^$/d' | sort -u
  )
  if [ -n "$files" ]; then
    archive_tip=true
    while IFS= read -r f; do
      [ -z "$f" ] && continue
      case "$f" in
        .specsync/archive/*|.specsync/change-sequence.json) ;;
        *) archive_tip=false; break ;;
      esac
    done <<<"$files"
  fi
fi

step_start=$(ts)
echo "==> [1/3] cargo fmt --check"
cargo fmt -- --check
echo "    ok ($(( $(ts) - step_start ))s)"

step_start=$(ts)
echo "==> [2/3] cargo check"
cargo check --quiet
echo "    ok ($(( $(ts) - step_start ))s)"

if [ "$archive_tip" = true ]; then
  echo "==> [3/3] archive-tip mode — skip product path coverage (CI archive-integrity)"
else
  step_start=$(ts)
  if [ -x target/release/specsync ]; then
    SPECSYNC_BIN=target/release/specsync
  elif [ -x target/debug/specsync ]; then
    SPECSYNC_BIN=target/debug/specsync
  else
    echo "    building release binary once…"
    cargo build --release --quiet
    SPECSYNC_BIN=target/release/specsync
  fi
  echo "==> [3/3] strict path/spec coverage"
  echo "    using $SPECSYNC_BIN"
  "$SPECSYNC_BIN" check --strict --require-coverage 100 --force
  echo "    ok ($(( $(ts) - step_start ))s)"
fi

total=$(( $(ts) - total_start ))
echo "==> pre-push gate PASS in ${total}s — safe to git push"
[ "$archive_tip" = true ] && echo "    archive-tip: CI should run archive-integrity"
echo "    (full test+clippy: fledge lanes run verify)"

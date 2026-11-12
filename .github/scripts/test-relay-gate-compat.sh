#!/usr/bin/env bash
# REQ-change-158 (#1064): a change that holds a relayed decision is recorded at workflow version
# 3, so the real SpecSync 6.2.0 release binary refuses it with "was written by a newer SpecSync"
# instead of rewriting it and dropping `relayed_by`.
#
# The binary under test records a relayed approval, then a relayed review, on a workflow-v2
# change. The 6.2.0 release asset, pinned by sha256, must then:
# - refuse every change command on that change with the newer-SpecSync message;
# - leave every file under `.specsync/` and the Git status byte-identical.
# With the change archived, 6.2.0 must also:
# - refuse `change new` and `change audit` the same way and change nothing;
# - still pass `check --strict`, which is all the Action and Trust run.
# Negative control: the same relayed approval with the version put back to 2 is what 6.2.0
# rewrites, and that rewrite drops `relayed_by`. This is the loss the gate prevents.
#
# Usage: .github/scripts/test-relay-gate-compat.sh <specsync binary under test>
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <specsync binary under test>" >&2
  exit 2
fi
NEW="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
[[ -x "$NEW" ]] || { echo "error: $NEW is not executable" >&2; exit 2; }

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)
    ASSET=specsync-linux-x86_64
    PINNED=be0c83a9676f7ae1116e5bd744e69a7b1cb8eabed767c1cbc455d7185c8e4c39
    ;;
  Darwin-arm64)
    ASSET=specsync-macos-aarch64
    PINNED=beb39b0e6f503abc1e18fa1527c5a3db199e21bd7cc17283714db7e3227acca9
    ;;
  *) echo "error: no pinned SpecSync 6.2.0 asset for $(uname -s)-$(uname -m)" >&2; exit 2 ;;
esac

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
failures=0
fail() {
  echo "FAIL: $*" >&2
  failures=$((failures + 1))
}

mkdir -p "$WORK/bin"
curl -fsSL --retry 3 -o "$WORK/bin/$ASSET.tar.gz" \
  "https://github.com/CorvidLabs/spec-sync/releases/download/v6.2.0/$ASSET.tar.gz"
actual="$(sha256_of "$WORK/bin/$ASSET.tar.gz")"
if [[ "$actual" != "$PINNED" ]]; then
  echo "error: 6.2.0/$ASSET.tar.gz has sha256 $actual, pinned $PINNED" >&2
  exit 1
fi
tar xzf "$WORK/bin/$ASSET.tar.gz" -C "$WORK/bin"
OLD="$WORK/bin/$ASSET"
chmod +x "$OLD"
[[ "$("$OLD" --version)" == "specsync 6.2.0" ]] || { echo "error: $OLD is not 6.2.0" >&2; exit 1; }
echo "binary under test: $("$NEW" --version); released: $("$OLD" --version)"

NEWER='was written by a newer SpecSync (workflow version 3); upgrade specsync to read it'

# A workflow-v2 project with one spec, so `check --strict` has something to check, and a bare
# origin, so `change check --commit` and `change ship` have a remote default.
new_project() {
  local base="$1" root="$1/work"
  mkdir -p "$base"
  git -C "$base" init -q --bare -b main origin.git
  mkdir -p "$root/.specsync" "$root/src" "$root/specs/greet"
  git -C "$root" init -q -b main
  git -C "$root" config user.email compat@specsync.dev
  git -C "$root" config user.name "SpecSync compat"
  git -C "$root" remote add origin "$base/origin.git"
  printf '{"version": 2, "enabled": true, "require_change_for_meaningful_files": false, "meaningful_paths": [], "ignored_paths": [], "verification_commands": ["true"], "custom_artifacts": {}, "principles_file": null}\n' \
    >"$root/.specsync/sdd.json"
  printf 'export function greet() { return "hi"; }\n' >"$root/src/greet.ts"
  cat >"$root/specs/greet/greet.spec.md" <<'SPEC'
---
module: greet
version: 1
status: stable
files:
  - src/greet.ts
---

# Greet

## Purpose

Greets.

## Public API

| Export | Description |
|--------|-------------|
| `greet` | Returns a greeting |

## Invariants

1. Returns hi.

## Behavioral Examples

- greet() returns "hi".

## Error Cases

- None.

## Dependencies

- None.

## Change Log

| Date | Change |
|------|--------|
| 2026-11-11 | Initial |
SPEC
  printf 'Initial.\n' >"$root/README.md"
  commit_all "$root" base
  git -C "$root" push -q origin HEAD:main
  git -C "$root" fetch -q origin
  git -C "$root" remote set-head origin main >/dev/null 2>&1
}

# Commit everything, if there is anything: `change ship` commits its own archive.
commit_all() {
  git -C "$1" add -A
  git -C "$1" diff --cached --quiet || git -C "$1" commit -qm "$2"
}

# Draft a documentation change (description `$2`, path `$3`) in `$1` with the binary under test;
# print its ID.
draft() {
  local root="$1" description="$2" path="$3" id dir artifact
  printf '%s.\n' "$description" >"$root/$path"
  "$NEW" --root "$root" change new "$description" --kind documentation --path "$path" \
    --no-spec-change --rationale "Documentation only" >/dev/null 2>&1
  id="$(ls "$root/.specsync/changes")"
  [[ "$(wc -l <<<"$id")" -eq 1 ]] || { echo "error: expected one active change, found: $id" >&2; exit 1; }
  "$NEW" --root "$root" change answer "$id" acceptance_criteria "Readers can follow it" >/dev/null
  "$NEW" --root "$root" change answer "$id" public_contract no >/dev/null
  "$NEW" --root "$root" change answer "$id" architecture_risk no >/dev/null
  "$NEW" --root "$root" change answer "$id" constraints none >/dev/null 2>&1 || true
  dir="$root/.specsync/changes/$id"
  for artifact in $(python3 -c 'import json,sys; print(" ".join(json.load(open(sys.argv[1]))["selected_artifacts"]))' "$dir/state.json"); do
    case "$artifact" in
      plan) printf '# Plan\n\n## Approach\n\nDo the work.\n\n## Out of scope\n\nNothing else.\n\n## Steps\n\nLand it.\n\n## Risks\n\nnone\n\n## Constraints consulted\n\nnone\n' >"$dir/plan.md" ;;
      tasks) printf '# Tasks\n\n- [x] Write it.\n' >"$dir/tasks.md" ;;
      *) printf '# Done\n\nDocument evidence.\n' >"$dir/$artifact.md" ;;
    esac
  done
  echo "$id"
}

relay=(--words "Approve the guide" --where "team chat, 2026-11-11, relayed by the compat agent" --recorded-by compat-agent)

# `workflow_version origin` of a state file.
versions() {
  python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(d.get("workflow_version", 1), d.get("workflow_origin_version"))' "$1"
}

# Every file under `.specsync/` with its hash, plus the Git status: what 6.2.0 must not change.
snapshot() {
  local hasher=(shasum -a 256)
  command -v sha256sum >/dev/null 2>&1 && hasher=(sha256sum)
  (cd "$1" && git status --porcelain=v1 --untracked-files=all && find .specsync -type f -print0 | sort -z | xargs -0 "${hasher[@]}")
}

# Run 6.2.0 with `$@` in `$R`: it must exit non-zero with the newer-SpecSync message.
expect_newer_refusal() {
  local label="$1" output; shift
  if output="$("$OLD" --root "$R" "$@" 2>&1)"; then
    fail "6.2.0 $label: succeeded but must refuse: $output"
  elif [[ "$output" != *"$NEWER"* ]]; then
    fail "6.2.0 $label: refusal lacks '$NEWER': $output"
  fi
}

# ─── The gate: a relayed approval, then a relayed review ─────────────────────────────────────
new_project "$WORK/gate"
R="$WORK/gate/work"
A="$(draft "$R" "Write a guide" GUIDE.md)"
"$NEW" --root "$R" change approve "$A" --actor Leif "${relay[@]}" >/dev/null
[[ "$(versions "$R/.specsync/changes/$A/state.json")" == "3 2" ]] ||
  fail "a relayed approval left state.json at $(versions "$R/.specsync/changes/$A/state.json"), not '3 2'"
grep -q '"relayed_by"' "$R/.specsync/changes/$A/approvals.json" || fail "the approval carries no relayed_by"
commit_all "$R" "Approve the guide with a relayed decision"

before="$(snapshot "$R")"
echo "== 6.2.0 on the active relay-gated change"
expect_newer_refusal "change status" change status "$A"
expect_newer_refusal "change show" change show "$A"
expect_newer_refusal "change ship-status" change ship-status "$A"
expect_newer_refusal "change answer" change answer "$A" public_contract yes
expect_newer_refusal "change depend" change depend "$A" "$A"
expect_newer_refusal "change approve" change approve "$A" --actor Probe
expect_newer_refusal "change send-back" change send-back "$A" --actor Probe --reason probe
expect_newer_refusal "change check" change check "$A"
expect_newer_refusal "change check --commit" change check "$A" --commit
expect_newer_refusal "change review" change review "$A" --reviewer Probe
expect_newer_refusal "change finalize" change finalize "$A"
expect_newer_refusal "change ship" change ship "$A"
expect_newer_refusal "change reopen" change reopen "$A" --actor Probe --reason probe
expect_newer_refusal "change correct" change correct "$A" public_contract yes --actor Probe --reason probe
expect_newer_refusal "change correct-owner" change correct-owner "$A" --actor Probe --reason probe --path GUIDE.md --spec greet
expect_newer_refusal "change reconcile-history" change reconcile-history "$A" --actor Probe --reason probe
expect_newer_refusal "change list" change list
expect_newer_refusal "change audit" change audit
[[ "$(snapshot "$R")" == "$before" ]] || fail "6.2.0 changed the repository while refusing the active change"

# The binary under test carries the change on: the relayed review keeps the earlier verification
# current, and the archive keeps version 3.
"$NEW" --root "$R" change check "$A" --commit >/dev/null 2>&1 || fail "change check --commit refused the gated change"
"$NEW" --root "$R" change review "$A" --reviewer Leif --words "Ship the guide" \
  --where "team chat, 2026-11-11, relayed by the compat agent" --recorded-by compat-agent >/dev/null ||
  fail "a relayed review refused the gated change"
"$NEW" --root "$R" change ship "$A" >/dev/null 2>&1 || fail "change ship refused the gated change"
commit_all "$R" "Archive the guide"
ARCHIVED="$(ls -d "$R"/.specsync/archive/changes/*-"$A" 2>/dev/null || true)"
if [[ -z "$ARCHIVED" ]]; then
  fail "the gated change was not archived"
else
  for file in state.json accepted-state.json; do
    [[ "$(versions "$ARCHIVED/$file")" == "3 2" ]] ||
      fail "archived $file is at $(versions "$ARCHIVED/$file"), not '3 2'"
  done
  grep -q '"relayed_by"' "$ARCHIVED/review.json" || fail "the archived review carries no relayed_by"
fi
"$NEW" --root "$R" change audit >/dev/null 2>&1 || fail "the binary under test fails change audit on its own archive"

before="$(snapshot "$R")"
echo "== 6.2.0 beside the archived relay-gated change"
printf 'A faq.\n' >"$R/FAQ.md"
expect_newer_refusal "change new" change new "Write a faq" --kind documentation --path FAQ.md \
  --no-spec-change --rationale "Documentation only"
rm -f "$R/FAQ.md"
expect_newer_refusal "change status (archived)" change status "$A"
expect_newer_refusal "change audit (archived)" change audit
[[ "$(snapshot "$R")" == "$before" ]] || fail "6.2.0 changed the repository beside the archived change"
# `check` is what the Action and Trust run. It writes only its rebuildable hash cache.
if ! output="$("$OLD" --root "$R" check --strict 2>&1)"; then
  fail "6.2.0 check --strict must still pass beside the gated archive: $output"
fi

# ─── Negative control: without the gate, 6.2.0 drops the relay ──────────────────────────────
new_project "$WORK/control"
R="$WORK/control/work"
B="$(draft "$R" "Write a guide" GUIDE.md)"
"$NEW" --root "$R" change approve "$B" --actor Leif "${relay[@]}" >/dev/null
python3 - "$R/.specsync/changes/$B/state.json" <<'PY'
import json, sys
path = sys.argv[1]
state = json.load(open(path))
state["workflow_version"] = 2
open(path, "w").write(json.dumps(state, indent=2) + "\n")
PY
commit_all "$R" "Approve the guide, version put back to 2"
"$OLD" --root "$R" change send-back "$B" --actor Probe --reason "control: an older binary rewrites the change" >/dev/null 2>&1 ||
  fail "control: 6.2.0 refused to rewrite an ungated change, so the control shows nothing"
if grep -q '"relayed_by"' "$R/.specsync/changes/$B/approvals.json"; then
  fail "control: 6.2.0 kept relayed_by, so the hazard the gate prevents did not reproduce"
fi

if [[ "$failures" -ne 0 ]]; then
  echo "relay gate compatibility: $failures failure(s)" >&2
  exit 1
fi
echo "relay gate compatibility: SpecSync 6.2.0 refuses a relay-gated change, active or archived, and changes nothing; without the gate it drops relayed_by."

#!/usr/bin/env bash
# REQ-change-143, SHIP-12.g: a change closed late by `specsync doctor` does not trip up an older
# SpecSync, and that older SpecSync never counts it as checked.
#
# Real SpecSync 6.0.0 and 6.2.0 release binaries, pinned by sha256, are run against repositories
# after `specsync doctor --fix --close-late` from the binary under test. The site and Corvidinho
# CI pin 6.0.0. Two fixtures:
#
# 1. CorvidLabs/site's shape, built with 6.0.0:
#    - a stale workflow-v1 `accepted` change that blocks every later archive;
#    - a never-approved draft;
#    - an unmerged v1 change accepted with current evidence;
#    - an unmerged v2 finalize that the blocker stopped halfway (site #616).
#    Doctor closes the blocker and the draft late. The old binaries must then:
#    - pass `change audit`;
#    - leave both out of `change list`;
#    - report them not found in `change status`, `change ship-status` and `change reopen`;
#    - resume the stopped finalize and archive the v1 change.
# 2. A real numbered collision, built with 5.2.0, the last release that minted `CHG-NNNN`
#    ordinals: two clones each mint CHG-0001, one is archived, the other stays accepted, and the
#    collision is acknowledged. Doctor closes the accepted member late and drops its
#    acknowledgement. The old binaries must then pass `change audit`.
#
# Usage: .github/scripts/test-closed-late-compat.sh <specsync binary under test>
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <specsync binary under test>" >&2
  exit 2
fi
NEW="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
[[ -x "$NEW" ]] || { echo "error: $NEW is not executable" >&2; exit 2; }

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) ASSET=specsync-linux-x86_64 ;;
  Darwin-arm64) ASSET=specsync-macos-aarch64 ;;
  *) echo "error: no pinned SpecSync asset for $(uname -s)-$(uname -m)" >&2; exit 2 ;;
esac

# The sha256 of each release asset, copied from the release's own .sha256 files.
pinned_sha256() {
  case "$1/$2" in
    5.2.0/specsync-linux-x86_64) echo 96f81829256ba769446db195db7ffd1822f53bd5530b2990c7b867805e34970e ;;
    5.2.0/specsync-macos-aarch64) echo 3061040b4c5913eb360b577dd2f2b2f2cef149a411993f91ee682b8c6331acb8 ;;
    6.0.0/specsync-linux-x86_64) echo 17ce857e9c48e3a23fa6767d0bbad7b7425c0095e0f5953cac67bf0f4f25aaec ;;
    6.0.0/specsync-macos-aarch64) echo 1328beeeba827a380840d51525e7aef1c67fee0ee53263db156948b3a29f7c42 ;;
    6.2.0/specsync-linux-x86_64) echo be0c83a9676f7ae1116e5bd744e69a7b1cb8eabed767c1cbc455d7185c8e4c39 ;;
    6.2.0/specsync-macos-aarch64) echo beb39b0e6f503abc1e18fa1527c5a3db199e21bd7cc17283714db7e3227acca9 ;;
    *) return 1 ;;
  esac
}

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

# Download one release binary and check it against its pinned sha256; print its path.
release_binary() {
  local version="$1" dir="$WORK/bin/$1" expected actual
  expected="$(pinned_sha256 "$version" "$ASSET")" || { echo "error: no pin for $version/$ASSET" >&2; exit 2; }
  mkdir -p "$dir"
  curl -fsSL --retry 3 -o "$dir/$ASSET.tar.gz" \
    "https://github.com/CorvidLabs/spec-sync/releases/download/v$version/$ASSET.tar.gz"
  actual="$(sha256_of "$dir/$ASSET.tar.gz")"
  if [[ "$actual" != "$expected" ]]; then
    echo "error: $version/$ASSET.tar.gz has sha256 $actual, pinned $expected" >&2
    exit 1
  fi
  tar xzf "$dir/$ASSET.tar.gz" -C "$dir"
  chmod +x "$dir/$ASSET"
  echo "$dir/$ASSET"
}

V520="$(release_binary 5.2.0)"
V600="$(release_binary 6.0.0)"
V620="$(release_binary 6.2.0)"
echo "binary under test: $("$NEW" --version)"

git_quiet() { git "$@" >/dev/null 2>&1; }

new_repo() {
  local root="$1"
  mkdir -p "$root/.specsync"
  git -C "$root" init -q -b main
  git -C "$root" config user.email compat@specsync.dev
  git -C "$root" config user.name "SpecSync compat"
}

write_sdd() {
  printf '{"version": %s, "enabled": true, "require_change_for_meaningful_files": false, "meaningful_paths": [], "ignored_paths": [], "verification_commands": ["true"], "custom_artifacts": {}, "principles_file": null}\n' \
    "$2" >"$1/.specsync/sdd.json"
}

commit_all() { git -C "$1" add -A && git -C "$1" commit -qm "$2"; }

publish() {
  git -C "$1" push -q --force origin HEAD:refs/heads/main
  git -C "$1" fetch -q origin
  git_quiet -C "$1" remote set-head origin main
}

# Fill every selected artifact of change `$3` in `$1` so it can be approved.
fill_artifacts() {
  local root="$1" id="$3" dir artifact
  dir="$root/.specsync/changes/$id"
  for artifact in $(python3 -c 'import json,sys; print(" ".join(json.load(open(sys.argv[1]))["selected_artifacts"]))' "$dir/state.json"); do
    case "$artifact" in
      plan) printf '# Plan\n\n## Approach\n\nDo the work.\n\n## Out of scope\n\nNothing else.\n\n## Steps\n\nLand it.\n\n## Risks\n\nnone\n\n## Constraints consulted\n\nnone\n' >"$dir/plan.md" ;;
      tasks) printf '# Tasks\n\n- [x] Write it.\n' >"$dir/tasks.md" ;;
      *) printf '# Done\n\nDocument evidence.\n' >"$dir/$artifact.md" ;;
    esac
  done
}

# Draft a change (description `$3`, path `$5`) in `$2` with binary `$1`; print its ID, which is the
# workspace `change new` created. `$4` is the slug the ID must contain.
draft() {
  local bin="$1" root="$2" description="$3" path="$5" id output before
  printf '%s.\n' "$description" >"$root/$path"
  before="$(ls "$root/.specsync/changes" 2>/dev/null || true)"
  output="$("$bin" --root "$root" change new "$description" --kind documentation --path "$path" \
    --no-spec-change --rationale "Documentation only" 2>&1)" || true
  id="$(ls "$root/.specsync/changes" 2>/dev/null | grep -vxF -e "$before" | grep -F -- "$4" | head -1 || true)"
  [[ -n "$id" ]] || { echo "error: no change minted for '$description': $output" >&2; exit 1; }
  "$bin" --root "$root" change answer "$id" acceptance_criteria "Readers can follow it" >/dev/null
  "$bin" --root "$root" change answer "$id" public_contract no >/dev/null
  "$bin" --root "$root" change answer "$id" architecture_risk no >/dev/null
  if python3 -c 'import json,sys; sys.exit(0 if json.load(open(sys.argv[1])).get("workflow_version") == 2 else 1)' \
    "$root/.specsync/changes/$id/state.json"; then
    "$bin" --root "$root" change answer "$id" constraints none >/dev/null
  fi
  fill_artifacts "$root" "$bin" "$id"
  echo "$id"
}

v1_accept() {
  local bin="$1" root="$2" id="$3"
  "$bin" --root "$root" change approve "$id" --actor Definer >/dev/null
  "$bin" --root "$root" change start "$id" >/dev/null
  "$bin" --root "$root" change verify "$id" >/dev/null
  "$bin" --root "$root" change accept "$id" --actor Closer >/dev/null
}

expect_ok() {
  local label="$1"; shift
  local output
  if ! output="$("$@" 2>&1)"; then
    fail "$label: exited non-zero: $output"
  fi
}

expect_refusal() {
  local label="$1" needle="$2"; shift 2
  local output
  if output="$("$@" 2>&1)"; then
    fail "$label: succeeded but must refuse: $output"
  elif [[ "$output" != *"$needle"* ]]; then
    fail "$label: refusal lacks '$needle': $output"
  fi
}

expect_absent() {
  local label="$1" needle="$2"; shift 2
  local output
  output="$("$@" 2>&1)" || fail "$label: exited non-zero: $output"
  if [[ "$output" == *"$needle"* ]]; then
    fail "$label: still lists '$needle': $output"
  fi
}

# ─── Fixture 1: the site's shape, built with 6.0.0 ────────────────────────────────────────────
SITE="$WORK/site"
mkdir -p "$SITE"
git -C "$SITE" init -q --bare -b main origin.git
new_repo "$SITE/work"
R="$SITE/work"
git -C "$R" remote add origin "$SITE/origin.git"
write_sdd "$R" 1
printf 'Initial.\n' >"$R/README.md"
commit_all "$R" base
BLOCKER="$(draft "$V600" "$R" "Update the readme" update-the-readme README.md)"
v1_accept "$V600" "$R" "$BLOCKER"
commit_all "$R" "Accept the readme change"
printf 'Rewritten later.\n' >"$R/README.md"
commit_all "$R" "Rewrite the readme"
DRAFT="$(draft "$V600" "$R" "Write a faq" write-a-faq FAQ.md)"
commit_all "$R" "Draft the FAQ change"
# A v1 change accepted with current evidence, whose input is edited for a while: doctor must leave a
# decision it was not given, and once the edit is reverted an old binary archives it itself.
CURRENT="$(draft "$V600" "$R" "Add notes" add-notes NOTES.md)"
v1_accept "$V600" "$R" "$CURRENT"
commit_all "$R" "Accept the notes change"
printf 'Notes, edited for a while.\n' >"$R/NOTES.md"
commit_all "$R" "Edit the notes"
write_sdd "$R" 2
commit_all "$R" "Adopt workflow v2"
publish "$R"
git -C "$R" checkout -q -b feature
STOPPED="$(draft "$V600" "$R" "Add a guide" add-a-guide GUIDE.md)"
"$V600" --root "$R" change approve "$STOPPED" --actor Definer >/dev/null
commit_all "$R" "Approve the guide"
"$V600" --root "$R" change check "$STOPPED" --commit >/dev/null 2>&1
"$V600" --root "$R" change review "$STOPPED" --reviewer Reviewer >/dev/null
if "$V600" --root "$R" change finalize "$STOPPED" >/dev/null 2>&1; then
  fail "fixture: the stopped finalize was not stopped by the blocker"
fi
commit_all "$R" "Keep the stopped finalize (site #616)"

# The late close, by the binary under test, on main where both changes merged.
git -C "$R" checkout -q main
expect_ok "doctor --close-late" "$NEW" --root "$R" doctor --fix \
  --close-late "$BLOCKER" --close-late "$DRAFT" --actor compat --reason "merged long ago, never archived"
for id in "$BLOCKER" "$DRAFT"; do
  ls -d "$R/.specsync/archive/closed-late/"*"-$id" >/dev/null 2>&1 || fail "doctor did not close $id late"
done
[[ -d "$R/.specsync/changes/$CURRENT" ]] || fail "doctor touched $CURRENT, a decision it was not given"
commit_all "$R" "doctor: close the blocker and the draft late"
printf 'Add notes.\n' >"$R/NOTES.md"
commit_all "$R" "Revert the notes edit"
publish "$R"
git -C "$R" checkout -q feature
git_quiet -C "$R" merge -q --no-edit main

for OLD in "$V600" "$V620"; do
  version="$("$OLD" --version)"
  copy="$WORK/site-$(echo "$version" | tr -c 'a-z0-9.' '-')"
  cp -R "$SITE" "$copy"
  C="$copy/work"
  git -C "$C" remote set-url origin "$copy/origin.git"
  echo "== $version on the site fixture"
  expect_ok "$version change audit" "$OLD" --root "$C" change audit
  expect_absent "$version change list" "$BLOCKER" "$OLD" --root "$C" change list
  expect_absent "$version change list" "$DRAFT" "$OLD" --root "$C" change list
  for id in "$BLOCKER" "$DRAFT"; do
    expect_refusal "$version change status $id" "not found" "$OLD" --root "$C" change status "$id"
    expect_refusal "$version change ship-status $id" "not found" "$OLD" --root "$C" change ship-status "$id"
  done
  expect_refusal "$version change reopen" "not found" \
    "$OLD" --root "$C" change reopen "$BLOCKER" --actor compat --reason probe
  expect_ok "$version change finalize (stopped finalize)" "$OLD" --root "$C" change finalize "$STOPPED"
  expect_ok "$version change archive (v1)" "$OLD" --root "$C" change archive "$CURRENT"
  expect_ok "$version change audit after" "$OLD" --root "$C" change audit
done

# The binary under test still reports what it closed late.
status="$("$NEW" --root "$R" change status "$BLOCKER" 2>&1)" || fail "new change status of $BLOCKER: $status"
[[ "$status" == *"closed late"* && "$status" == *"not verified"* ]] || fail "new change status: $status"

# ─── Fixture 2: a real CHG-0001 collision, built with 5.2.0 ─────────────────────────────────────
COLL="$WORK/collision"
mkdir -p "$COLL"
git -C "$COLL" init -q --bare -b main origin.git
new_repo "$COLL/a"
A="$COLL/a"
git -C "$A" remote add origin "$COLL/origin.git"
write_sdd "$A" 1
printf 'Base.\n' >"$A/README.md"
commit_all "$A" base
git -C "$A" push -q origin HEAD:main
git clone -q "$COLL/origin.git" "$COLL/b"
B="$COLL/b"
git -C "$B" config user.email compat@specsync.dev
git -C "$B" config user.name "SpecSync compat"
ARCHIVED="$(draft "$V520" "$A" "Write the guide" write-the-guide GUIDE.md)"
v1_accept "$V520" "$A" "$ARCHIVED"
"$V520" --root "$A" change archive "$ARCHIVED" >/dev/null
commit_all "$A" "Archive the guide"
git -C "$A" push -q origin HEAD:main
MEMBER="$(draft "$V520" "$B" "Write the faq" write-the-faq FAQ.md)"
v1_accept "$V520" "$B" "$MEMBER"
commit_all "$B" "Accept the faq"
[[ "${ARCHIVED%%-write*}" == "${MEMBER%%-write*}" ]] || fail "fixture: $ARCHIVED and $MEMBER do not collide"
git_quiet -C "$B" pull -q --no-rebase --no-edit origin main || true
python3 - "$B/.specsync/change-sequence.json" "$ARCHIVED" "$MEMBER" <<'PY'
import json, sys
path, archived, member = sys.argv[1:]
ledger = {
    "schema_version": 1,
    "sequence": 1,
    "id": archived,
    "acknowledged_collisions": [{"sequence": 1, "ids": sorted([archived, member])}],
}
open(path, "w").write(json.dumps(ledger, indent=2) + "\n")
PY
commit_all "$B" "Acknowledge the CHG-0001 collision"
publish "$B"
expect_ok "doctor --close-late (collision member)" "$NEW" --root "$B" doctor --fix \
  --close-late "$MEMBER" --actor compat --reason "merged long ago, never archived"
if ! python3 -c 'import json,sys; sys.exit(0 if json.load(open(sys.argv[1]))["acknowledged_collisions"] == [] else 1)' \
  "$B/.specsync/change-sequence.json"; then
  fail "doctor did not drop the CHG-0001 acknowledgement"
fi
commit_all "$B" "doctor: close the collision member late"
for OLD in "$V600" "$V620"; do
  echo "== $("$OLD" --version) on the collision fixture"
  expect_ok "$("$OLD" --version) change audit" "$OLD" --root "$B" change audit
  expect_refusal "$("$OLD" --version) change status" "not found" "$OLD" --root "$B" change status "$MEMBER"
done

if [[ "$failures" -ne 0 ]]; then
  echo "closed-late compatibility: $failures failure(s)" >&2
  exit 1
fi
echo "closed-late compatibility: SpecSync 6.0.0 and 6.2.0 read the repositories doctor wrote and never count a closed-late change as checked."

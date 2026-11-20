#!/bin/bash
# run.sh <new specsync> <old specsync> <work dir>
#
# What released SpecSync 6.3.0 does in a repository where 6.4.0 (main) has written history
# fragments, for the 6.4.0 upgrade note. A PR-shaped fixture: origin/main exists, a change to the
# `auth` module is made on a branch with the new binary (as tests/integration/spec_history.rs
# does, but with require_change_for_meaningful_files on, as in spec-sync itself), then each binary
# is run at three tips: the product tip after `change check --commit`, the archive tip after
# `change finalize`, and main after the merge.
set -u
NEW=$1 OLD=$2 W=$3
rm -rf "$W" && mkdir -p "$W" && cd "$W" || exit 2
git init -q --bare origin.git
git init -q -b main repo && cd repo || exit 2
for kv in user.email=test@example.com user.name=Test commit.gpgsign=false tag.gpgsign=false core.autocrlf=false core.eol=lf; do
  git config "${kv%%=*}" "${kv#*=}"
done
put() { mkdir -p "$(dirname "$1")"; printf '%s' "$2" > "$1"; }
CHANGE_LOG=$'## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n'
put src/auth.rs $'// Authentication module.\n'
put specs/auth/auth.spec.md $'---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n'"$CHANGE_LOG"
put specs/auth/requirements.md $'---\nspec: auth.spec.md\n---\n\n# Requirements\n\n### REQ-auth-001\n\nThe system SHALL keep one session per user.\n\nAcceptance Criteria\n- One session per user.\n'
put .gitignore $'.specsync/hashes.json\n'
put .specsync/sdd.json '{
  "version": 2,
  "enabled": true,
  "require_change_for_meaningful_files": true,
  "meaningful_paths": ["src/", ".specsync/sdd.json"],
  "ignored_paths": [],
  "verification_commands": ["true"],
  "custom_artifacts": {},
  "principles_file": null
}
'
git add --all && git commit -q -m base && git remote add origin ../origin.git && git push -q origin main && git fetch -q origin

# Run one binary and print its exit code and the lines that say what happened.
say() {
  local who=$1; shift
  local bin=$NEW; [ "$who" = old ] && bin=$OLD
  echo "\$ $who($("$bin" --version | cut -d' ' -f2)) specsync $*"
  "$bin" "$@" < /dev/null > ../out.txt 2>&1
  local code=$?
  grep -vE '^\s*$' ../out.txt | head -${LINES_SHOWN:-12} | sed 's/^/    /'
  echo "    -> exit $code"
}

echo "## The new binary ships a change to auth on branch passkeys"
git checkout -q -b passkeys
ID=$("$NEW" --json change new "Let a user sign in with a passkey" --kind feature --spec auth --path src/auth.rs | python3 -I -c 'import json,sys; print(json.load(sys.stdin)["change"]["id"])')
"$NEW" change answer "$ID" acceptance_criteria "The auth contract says it" > /dev/null
"$NEW" change answer "$ID" public_contract no > /dev/null
"$NEW" change answer "$ID" architecture_risk no > /dev/null
WS=.specsync/changes/$ID
for name in $(python3 -I -c 'import json,sys; print(" ".join(json.load(open(sys.argv[1]))["selected_artifacts"]))' "$WS/state.json"); do
  case $name in
    tasks) put "$WS/tasks.md" $'# Tasks\n\n- [x] Say it in the contract.\n' ;;
    plan) put "$WS/plan.md" $'# Plan\n\n## Approach\n\nDo the work.\n\n## Out of scope\n\nLeave unrelated work alone.\n\n## Steps\n\nLand the change.\n\n## Risks\n\nnone\n\n## Constraints consulted\n\nnone\n' ;;
    testing) put "$WS/testing.md" $'# Testing\n\n## Requirement evidence\n\n| Requirement | Evidence |\n|---|---|\n| REQ-auth-002 | SpecSync structural check of the module |\n' ;;
    *) put "$WS/$name.md" $'# Complete\n\nReviewed contract wording.\n' ;;
  esac
done
put "$WS/deltas/auth.md" $'## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL let a user sign in with a passkey.\n\nAcceptance Criteria\n- A passkey signs the user in.\n'
"$NEW" change approve "$ID" --actor "Scope owner" > /dev/null && git add --all && git commit -q -m "approve $ID"
say new change check "$ID" --commit
echo "    files the branch adds or changes under specs/: $(git diff --name-only origin/main...HEAD -- specs | tr '\n' ' ')"
echo "    spec version line: $(grep '^version:' specs/auth/auth.spec.md); Change Log rows: $(grep -c '^| 20' specs/auth/auth.spec.md)"

echo; echo "## Product tip (the change is open, its fragment committed)"
for who in old new; do say $who check; done
for who in old new; do say $who change audit; done
for who in old new; do say $who change audit --strict; done

echo; echo "## Released 6.3.0 runs change check on the change the new binary recorded (no --commit)"
say old change check "$ID"
echo "    spec version line now: $(grep '^version:' specs/auth/auth.spec.md); Change Log rows: $(grep -c '^| 20' specs/auth/auth.spec.md); new row: $(grep "$ID" specs/auth/auth.spec.md | head -1)"
echo "    fragment still there: $(test -f specs/auth/history/$ID.md && echo yes || echo no)"
say new check
git reset -q --hard && git clean -qfd
PRODUCT_TIP=$(git rev-parse HEAD)

echo; echo "## The same with --commit, as an agent runs it, and the way out"
say old change check "$ID" --commit
echo "    6.3.0 committed: $(git log -1 --format=%s); spec $(grep '^version:' specs/auth/auth.spec.md), Change Log rows $(grep -c '^| 20' specs/auth/auth.spec.md)"
say new check
echo "    way out, step 1: remove the Change Log row 6.3.0 added for $ID, keep the fragment"
grep -v "^| [0-9-]* | $ID: " specs/auth/auth.spec.md > ../spec.tmp && mv ../spec.tmp specs/auth/auth.spec.md
echo "    spec $(grep '^version:' specs/auth/auth.spec.md), Change Log rows $(grep -c '^| 20' specs/auth/auth.spec.md); diff from the product tip: $(git diff --stat "$PRODUCT_TIP" -- specs/auth/auth.spec.md | tail -1)"
say new check
echo "    way out, step 2: put version: back to the value before 6.3.0's bump"
sed 's/^version: 2$/version: 1/' specs/auth/auth.spec.md > ../spec.tmp && mv ../spec.tmp specs/auth/auth.spec.md
echo "    spec identical to the product tip: $(git diff --quiet "$PRODUCT_TIP" -- specs/auth/auth.spec.md && echo yes || echo no)"
say new check
git add --all && git commit -q -m "remove the Change Log row and version bump 6.3.0 added for $ID"
say new change status "$ID"
say new change check "$ID" --commit
say new change audit --strict
git reset -q --hard "$PRODUCT_TIP" && git clean -qfd

echo; echo "## Archive tip (review and finalize with the new binary, committed)"
"$NEW" change review "$ID" --reviewer Reviewer > /dev/null && "$NEW" change finalize "$ID" > /dev/null && git add --all && git commit -q -m "archive $ID"
echo "    open changes: $(ls .specsync/changes 2>/dev/null | wc -l | tr -d ' ')"
for who in old new; do say $who check; done
for who in old new; do say $who change audit; done
for who in old new; do say $who change audit --strict; done

echo; echo "## Main after the merge (origin/main at the merged tip)"
git checkout -q main && git merge -q --ff-only passkeys && git push -q origin main && git fetch -q origin
for who in old new; do say $who change audit --strict; done
say old check

echo; echo "## change approve --dry-run --json, on a change with no acceptance criterion yet"
git checkout -q -b dry-run
"$NEW" --json change new "Try the dry run" --kind documentation --path NOTES.md --no-spec-change --rationale "Documentation only" > /dev/null
LINES_SHOWN=90 say new --json change approve try-the-dry-run --dry-run

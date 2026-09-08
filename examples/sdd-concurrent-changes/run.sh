#!/usr/bin/env bash
set -euo pipefail

bin="${SPECSYNC_BIN:-specsync}"
root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT

cd "$root"
git init -b main >/dev/null
git config user.email example@specsync.dev
git config user.name "SpecSync Example"
printf '# Ordered changes\n' > README.md
"$bin" init >/dev/null
git add .
git commit -m "Initialize example" >/dev/null

create_change() {
  local description="$1"
  local path="$2"
  local output_name="$3"
  local created
  local id
  created="$("$bin" change new "$description" --kind operations --path "$path" \
    --no-spec-change --rationale "Operational ordering only" --json)"
  id="$(printf '%s' "$created" | python3 -c 'import json,sys; print(json.load(sys.stdin)["change"]["id"])')"
  printf -v "$output_name" '%s' "$id"
  "$bin" change answer "$id" acceptance_criteria "$description is complete" >/dev/null
  "$bin" change answer "$id" public_contract no >/dev/null
  "$bin" change answer "$id" architecture_risk no >/dev/null
  local dir=".specsync/changes/$id"
  printf '# Context\n\n%s\n' "$description" > "$dir/context.md"
  printf '# Plan\n\nExecute in declared order.\n' > "$dir/plan.md"
  printf '# Testing\n\nOrdering is verified by lifecycle gates.\n' > "$dir/testing.md"
}

create_change "Deploy dependent service" "ops/service/" first
create_change "Provision prerequisite" "ops/platform/" second
"$bin" change depend "$first" "$second" >/dev/null
"$bin" change approve "$first" --actor "Example Scope Owner" >/dev/null
"$bin" change approve "$second" --actor "Example Scope Owner" >/dev/null

if "$bin" change check "$first" >/dev/null 2>&1; then
  echo "dependent change verified before prerequisite" >&2
  exit 1
fi

"$bin" change check "$second" >/dev/null
"$bin" change review "$second" --reviewer "Example Independent Reviewer" >/dev/null
"$bin" change finalize "$second" >/dev/null
"$bin" change check "$first" >/dev/null
"$bin" change review "$first" --reviewer "Example Independent Reviewer" >/dev/null
"$bin" change finalize "$first" >/dev/null
git add .
git commit -m "Complete ordered changes" >/dev/null
"$bin" change audit

printf '\nConcurrent-change example passed in %s\n' "$root"

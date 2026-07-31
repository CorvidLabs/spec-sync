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
# Operations-only projects need a bounded fallback so scoped check can record evidence.
python3 - <<'PY'
import json
from pathlib import Path

path = Path(".specsync/sdd.json")
policy = json.loads(path.read_text())
if not policy.get("verification_commands"):
    policy["verification_commands"] = ["true"]
    path.write_text(json.dumps(policy, indent=2) + "\n")
PY
git add .
git commit -m "Initialize example" >/dev/null

create_change() {
  local description="$1"
  local path="$2"
  local id="$3"
  "$bin" change new "$description" --kind operations --path "$path" \
    --no-spec-change --rationale "Operational ordering only" >/dev/null
  "$bin" change answer "$id" acceptance_criteria "$description is complete" >/dev/null
  "$bin" change answer "$id" public_contract no >/dev/null
  "$bin" change answer "$id" architecture_risk no >/dev/null
  local dir=".specsync/changes/$id"
  printf '# Context\n\n%s\n' "$description" > "$dir/context.md"
  printf '# Plan\n\nExecute in declared order.\n' > "$dir/plan.md"
  printf '# Testing\n\nOrdering is verified by lifecycle gates.\n' > "$dir/testing.md"
}

first="CHG-0001-deploy-dependent-service"
second="CHG-0002-provision-prerequisite"
create_change "Deploy dependent service" "ops/service/" "$first"
create_change "Provision prerequisite" "ops/platform/" "$second"
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

#!/usr/bin/env bash
set -euo pipefail

bin="${SPECSYNC_BIN:-specsync}"
root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT

cd "$root"
git init -b main >/dev/null
git config user.email example@specsync.dev
git config user.name "SpecSync Example"
printf '# Example project\n' > README.md
"$bin" init >/dev/null
git add .
git commit -m "Initialize example" >/dev/null

created="$("$bin" change new "Clarify contributor workflow" \
  --kind documentation \
  --path README.md \
  --no-spec-change \
  --rationale "Documentation wording does not alter the technical contract" --json)"
id="$(printf '%s' "$created" | python3 -c 'import json,sys; print(json.load(sys.stdin)["change"]["id"])')"
"$bin" change answer "$id" acceptance_criteria \
  "Contributors can follow the documented workflow" >/dev/null
"$bin" change answer "$id" public_contract no >/dev/null
"$bin" change answer "$id" architecture_risk no >/dev/null

dir=".specsync/changes/$id"
printf '# Context\n\nClarify the contributor workflow.\n' > "$dir/context.md"
printf '# Docs\n\nThe reviewed workflow is executable.\n' > "$dir/docs.md"

"$bin" change approve "$id" --actor "Example Scope Owner" >/dev/null
printf '\nFollow the verified SDD lifecycle.\n' >> README.md
"$bin" change check "$id"
git add .
git commit -m "Clarify contributor workflow" >/dev/null
"$bin" change check "$id"
"$bin" change review "$id" --reviewer "Example Independent Reviewer" >/dev/null
"$bin" change finalize "$id" >/dev/null
git add .
git commit -m "Finalize contributor workflow archive" >/dev/null
"$bin" change audit

printf '\nLifecycle example passed in %s\n' "$root"

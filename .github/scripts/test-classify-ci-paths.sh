#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
classifier="$script_dir/classify-ci-paths.sh"
fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT

mkdir -p "$fixture/.specsync/archive/changes/CHG-0001" \
    "$fixture/.specsync/changes/CHG-0002" \
    "$fixture/site/src" \
    "$fixture/vscode-extension/src"
touch "$fixture/.specsync/archive/changes/CHG-0001/state.json"
touch "$fixture/.specsync/changes/CHG-0002/state.json"

classify() {
    local output
    output="$(printf '%s\0' "$@" | "$classifier" "$fixture")"
    printf '%s\n' "$output"
}

expect_value() {
    local output="$1" key="$2" expected="$3"
    if ! grep -Fxq "$key=$expected" <<<"$output"; then
        printf 'expected %s=%s, got:\n%s\n' "$key" "$expected" "$output" >&2
        exit 1
    fi
}

archive_move="$(classify \
    .specsync/changes/CHG-0001/state.json \
    .specsync/archive/changes/CHG-0001/state.json)"
expect_value "$archive_move" archive_only true
expect_value "$archive_move" full false

active_edit="$(classify .specsync/changes/CHG-0002/state.json)"
expect_value "$active_edit" archive_only false

source_change="$(classify src/parser.rs)"
expect_value "$source_change" full true

site_change="$(classify site/src/pages/index.astro)"
expect_value "$site_change" full false
expect_value "$site_change" site true

vscode_change="$(classify vscode-extension/src/extension.ts)"
expect_value "$vscode_change" full false
expect_value "$vscode_change" vscode true

contract_change="$(classify specs/parser/parser.spec.md)"
expect_value "$contract_change" archive_only false
expect_value "$contract_change" full false

mixed_change="$(classify \
    .specsync/archive/changes/CHG-0001/state.json \
    site/src/pages/index.astro)"
expect_value "$mixed_change" archive_only false
expect_value "$mixed_change" site true

workflow_change="$(classify .github/workflows/ci.yml)"
expect_value "$workflow_change" full true

forced="$(printf '' | "$classifier" "$fixture" true)"
expect_value "$forced" full true

echo "classify-ci-paths tests passed"

#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
classifier="$script_dir/classify-ci-paths.sh"
fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT

mkdir -p "$fixture/.specsync/archive/changes/CHG-0001" \
    "$fixture/.specsync/archive/changes/2026-07-29-CHG-0001-finalize-widget" \
    "$fixture/.specsync/archive/changes/2026-07-29-CHG-0003-other-change" \
    "$fixture/.specsync/archive/changes/2026-07-29-CHG-10000-large-sequence" \
    "$fixture/.specsync/changes/CHG-0002-active-widget" \
    "$fixture/site/src" \
    "$fixture/vscode-extension/src"
touch "$fixture/.specsync/archive/changes/2026-07-29-CHG-0001-finalize-widget/state.json"
touch "$fixture/.specsync/archive/changes/2026-07-29-CHG-0003-other-change/state.json"
touch "$fixture/.specsync/archive/changes/2026-07-29-CHG-10000-large-sequence/state.json"
printf '%s\n' \
    '{"id":"CHG-0002-active-widget","state":"verifying"}' \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/state.json"
printf '%s\n' \
    '{"passed":true,"contract_digest":"contract","workspace_digest":"current"}' \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/verification.json"
printf '%s\n' \
    '{"change_id":"CHG-0002-active-widget","contract_digest":"contract","workspace_digest":"stale"}' \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/review.json"

classify() {
    local output
    output="$(printf '%s\0' "$@" | "$classifier" "$fixture" false name-status)"
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
    D .specsync/changes/CHG-0001-finalize-widget/state.json \
    A .specsync/archive/changes/2026-07-29-CHG-0001-finalize-widget/state.json)"
expect_value "$archive_move" archive_only true
expect_value "$archive_move" full false
expect_value "$archive_move" archive_change_id CHG-0001-finalize-widget
expect_value "$archive_move" archive_dir 2026-07-29-CHG-0001-finalize-widget

rename_move="$(classify \
    R100 .specsync/changes/CHG-0001-finalize-widget/state.json \
    .specsync/archive/changes/2026-07-29-CHG-0001-finalize-widget/state.json)"
expect_value "$rename_move" archive_only true

large_change_id_move="$(classify \
    D .specsync/changes/CHG-10000-large-sequence/state.json \
    A .specsync/archive/changes/2026-07-29-CHG-10000-large-sequence/state.json)"
expect_value "$large_change_id_move" archive_only true
expect_value "$large_change_id_move" archive_change_id CHG-10000-large-sequence

active_edit="$(classify M .specsync/changes/CHG-0002-active-widget/state.json)"
expect_value "$active_edit" archive_only false
expect_value "$active_edit" review_only false

review_record="$(classify A .specsync/changes/CHG-0002-active-widget/review.json)"
expect_value "$review_record" review_only true
expect_value "$review_record" review_change_id CHG-0002-active-widget
expect_value "$review_record" full false

review_with_state="$(classify \
    A .specsync/changes/CHG-0002-active-widget/review.json \
    M .specsync/changes/CHG-0002-active-widget/state.json)"
expect_value "$review_with_state" review_only false

modified_review="$(classify M .specsync/changes/CHG-0002-active-widget/review.json)"
expect_value "$modified_review" review_only false

source_change="$(classify M src/parser.rs)"
expect_value "$source_change" full true
expect_value "$source_change" review_required true
expect_value "$source_change" review_required_change_id CHG-0002-active-widget

site_change="$(classify M site/src/pages/index.astro)"
expect_value "$site_change" full false
expect_value "$site_change" site true

vscode_change="$(classify M vscode-extension/src/extension.ts)"
expect_value "$vscode_change" full false
expect_value "$vscode_change" vscode true

contract_change="$(classify M specs/parser/parser.spec.md)"
expect_value "$contract_change" archive_only false
expect_value "$contract_change" full false

mixed_change="$(classify \
    D .specsync/changes/CHG-0001-finalize-widget/state.json \
    A .specsync/archive/changes/2026-07-29-CHG-0001-finalize-widget/state.json \
    M site/src/pages/index.astro)"
expect_value "$mixed_change" archive_only false
expect_value "$mixed_change" site true

wrong_archive="$(classify \
    D .specsync/changes/CHG-0001-finalize-widget/state.json \
    A .specsync/archive/changes/2026-07-29-CHG-0003-other-change/state.json)"
expect_value "$wrong_archive" archive_only false
expect_value "$wrong_archive" full true

archive_modification="$(classify \
    M .specsync/archive/changes/2026-07-29-CHG-0001-finalize-widget/state.json)"
expect_value "$archive_modification" archive_only false
expect_value "$archive_modification" full true

archive_without_active="$(classify \
    A .specsync/archive/changes/2026-07-29-CHG-0001-finalize-widget/finalization.json)"
expect_value "$archive_without_active" archive_only false
expect_value "$archive_without_active" full true

active_without_archive="$(classify \
    D .specsync/changes/CHG-0001-finalize-widget/state.json)"
expect_value "$active_without_archive" archive_only false

workflow_change="$(classify M .github/workflows/ci.yml)"
expect_value "$workflow_change" full true

forced="$(printf '' | "$classifier" "$fixture" true)"
expect_value "$forced" full true

name_only="$(printf '%s\0' \
    .specsync/changes/CHG-0001-finalize-widget/state.json \
    .specsync/archive/changes/2026-07-29-CHG-0001-finalize-widget/state.json \
    | "$classifier" "$fixture")"
expect_value "$name_only" archive_only false
expect_value "$name_only" review_only false
expect_value "$name_only" full true

printf '%s\n' \
    '{"change_id":"CHG-0002-active-widget","contract_digest":"contract","workspace_digest":"current"}' \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/review.json"
current_review="$(classify M specs/parser/parser.spec.md)"
expect_value "$current_review" review_required false

echo "classify-ci-paths tests passed"

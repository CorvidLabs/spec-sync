#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
classifier="$script_dir/classify-ci-paths.sh"
fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT

mkdir -p "$fixture/.specsync/archive/changes/CHG-0001" \
    "$fixture/.specsync/archive/changes/2026-07-29-CHG-0001-finalize-widget" \
    "$fixture/.specsync/archive/changes/2026-07-29-CHG-0003-other-change" \
    "$fixture/.specsync/archive/changes/2026-07-29-CHG-0004-legacy-change" \
    "$fixture/.specsync/archive/changes/2026-07-29-CHG-0005-legacy-without-version" \
    "$fixture/.specsync/archive/changes/2026-07-29-CHG-10000-large-sequence" \
    "$fixture/.specsync/changes/CHG-0002-active-widget" \
    "$fixture/.github/scripts" \
    "$fixture/site/src" \
    "$fixture/vscode-extension/src"
cp "$script_dir/lifecycle-validation-limits.json" "$fixture/.github/scripts/"
printf '%s\n' '{"id":"CHG-0001-finalize-widget","workflow_version":2}' \
    > "$fixture/.specsync/archive/changes/2026-07-29-CHG-0001-finalize-widget/state.json"
printf '%s\n' '{"id":"CHG-0003-other-change","workflow_version":2}' \
    > "$fixture/.specsync/archive/changes/2026-07-29-CHG-0003-other-change/state.json"
printf '%s\n' '{"id":"CHG-0004-legacy-change","workflow_version":1}' \
    > "$fixture/.specsync/archive/changes/2026-07-29-CHG-0004-legacy-change/state.json"
printf '%s\n' '{"id":"CHG-0005-legacy-without-version"}' \
    > "$fixture/.specsync/archive/changes/2026-07-29-CHG-0005-legacy-without-version/state.json"
printf '%s\n' '{"id":"CHG-10000-large-sequence","workflow_version":2}' \
    > "$fixture/.specsync/archive/changes/2026-07-29-CHG-10000-large-sequence/state.json"
printf '%s\n' \
    '{"id":"CHG-0002-active-widget","state":"verifying"}' \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/state.json"
printf '%s\n' \
    '{"passed":true,"contract_digest":"contract","execution_digest":"execution","workspace_digest":"current"}' \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/verification.json"
printf '%s\n' \
    '{"approvals":[{"gate":"definition","actor":"Scope owner","timestamp":1,"digest":"scope"}]}' \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/approvals.json"
printf '%s\n' \
    '{"change_id":"CHG-0002-active-widget","contract_digest":"contract","workspace_digest":"stale"}' \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/review.json"
printf '%s\n' \
    '{"schema_version":1,"reviews":[{"change_id":"CHG-0002-active-widget","contract_digest":"contract","workspace_digest":"stale"}]}' \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/review-attempts.json"
git -C "$fixture" init -q
git -C "$fixture" config user.email classifier@specsync.dev
git -C "$fixture" config user.name "SpecSync Classifier"
git -C "$fixture" add .
git -C "$fixture" commit -qm "fixture baseline"
fixture_head="$(git -C "$fixture" rev-parse HEAD)"

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
expect_value "$archive_move" archive_workflow_version 2
expect_value "$archive_move" legacy_archive_only false

rename_move="$(classify \
    R100 .specsync/changes/CHG-0001-finalize-widget/state.json \
    .specsync/archive/changes/2026-07-29-CHG-0001-finalize-widget/state.json)"
expect_value "$rename_move" archive_only true

large_change_id_move="$(classify \
    D .specsync/changes/CHG-10000-large-sequence/state.json \
    A .specsync/archive/changes/2026-07-29-CHG-10000-large-sequence/state.json)"
expect_value "$large_change_id_move" archive_only true
expect_value "$large_change_id_move" archive_change_id CHG-10000-large-sequence

unbound_legacy_archive_move="$(classify \
    D .specsync/changes/CHG-0004-legacy-change/state.json \
    A .specsync/archive/changes/2026-07-29-CHG-0004-legacy-change/state.json)"
expect_value "$unbound_legacy_archive_move" archive_only false
expect_value "$unbound_legacy_archive_move" legacy_archive_only false
expect_value "$unbound_legacy_archive_move" archive_workflow_version 1
expect_value "$unbound_legacy_archive_move" archive_parent_workflow_version ""
expect_value "$unbound_legacy_archive_move" full true

unbound_legacy_without_version="$(classify \
    D .specsync/changes/CHG-0005-legacy-without-version/state.json \
    A .specsync/archive/changes/2026-07-29-CHG-0005-legacy-without-version/state.json)"
expect_value "$unbound_legacy_without_version" archive_only false
expect_value "$unbound_legacy_without_version" legacy_archive_only false
expect_value "$unbound_legacy_without_version" archive_workflow_version 1
expect_value "$unbound_legacy_without_version" full true

legacy_fixture="$fixture/legacy-git"
mkdir -p "$legacy_fixture/.specsync/changes/CHG-0004-legacy-change" \
    "$legacy_fixture/.specsync/changes/CHG-0005-legacy-without-version" \
    "$legacy_fixture/.specsync/changes/CHG-0006-v2-downgrade"
mkdir -p "$legacy_fixture/.github/scripts"
cp "$script_dir/lifecycle-validation-limits.json" "$legacy_fixture/.github/scripts/"
printf '%s\n' '{"id":"CHG-0004-legacy-change","workflow_version":1}' \
    > "$legacy_fixture/.specsync/changes/CHG-0004-legacy-change/state.json"
printf '%s\n' '{"id":"CHG-0005-legacy-without-version"}' \
    > "$legacy_fixture/.specsync/changes/CHG-0005-legacy-without-version/state.json"
printf '%s\n' '{"id":"CHG-0006-v2-downgrade","workflow_version":2}' \
    > "$legacy_fixture/.specsync/changes/CHG-0006-v2-downgrade/state.json"
git -C "$legacy_fixture" init -q
git -C "$legacy_fixture" config user.email classifier@specsync.dev
git -C "$legacy_fixture" config user.name "SpecSync Classifier"
git -C "$legacy_fixture" add .
git -C "$legacy_fixture" commit -qm "fixture parent"
legacy_parent="$(git -C "$legacy_fixture" rev-parse HEAD)"
mkdir -p "$legacy_fixture/.specsync/archive/changes/2026-07-29-CHG-0004-legacy-change" \
    "$legacy_fixture/.specsync/archive/changes/2026-07-29-CHG-0005-legacy-without-version" \
    "$legacy_fixture/.specsync/archive/changes/2026-07-29-CHG-0006-v2-downgrade"
mv "$legacy_fixture/.specsync/changes/CHG-0004-legacy-change/state.json" \
    "$legacy_fixture/.specsync/archive/changes/2026-07-29-CHG-0004-legacy-change/state.json"
mv "$legacy_fixture/.specsync/changes/CHG-0005-legacy-without-version/state.json" \
    "$legacy_fixture/.specsync/archive/changes/2026-07-29-CHG-0005-legacy-without-version/state.json"
mv "$legacy_fixture/.specsync/changes/CHG-0006-v2-downgrade/state.json" \
    "$legacy_fixture/.specsync/archive/changes/2026-07-29-CHG-0006-v2-downgrade/state.json"
rmdir "$legacy_fixture/.specsync/changes/CHG-0004-legacy-change" \
    "$legacy_fixture/.specsync/changes/CHG-0005-legacy-without-version" \
    "$legacy_fixture/.specsync/changes/CHG-0006-v2-downgrade"
printf '%s\n' '{"id":"CHG-0006-v2-downgrade","workflow_version":1}' \
    > "$legacy_fixture/.specsync/archive/changes/2026-07-29-CHG-0006-v2-downgrade/state.json"

classify_bound_legacy() {
    printf '%s\0' "$@" \
        | "$classifier" "$legacy_fixture" false name-status "$legacy_parent"
}

legacy_archive_move="$(classify_bound_legacy \
    D .specsync/changes/CHG-0004-legacy-change/state.json \
    A .specsync/archive/changes/2026-07-29-CHG-0004-legacy-change/state.json)"
expect_value "$legacy_archive_move" archive_only false
expect_value "$legacy_archive_move" legacy_archive_only true
expect_value "$legacy_archive_move" archive_workflow_version 1
expect_value "$legacy_archive_move" archive_parent_workflow_version 1
expect_value "$legacy_archive_move" full true

legacy_without_version="$(classify_bound_legacy \
    D .specsync/changes/CHG-0005-legacy-without-version/state.json \
    A .specsync/archive/changes/2026-07-29-CHG-0005-legacy-without-version/state.json)"
expect_value "$legacy_without_version" archive_only false
expect_value "$legacy_without_version" legacy_archive_only true
expect_value "$legacy_without_version" archive_workflow_version 1
expect_value "$legacy_without_version" archive_parent_workflow_version 1
expect_value "$legacy_without_version" full true

v2_downgrade="$(classify_bound_legacy \
    D .specsync/changes/CHG-0006-v2-downgrade/state.json \
    A .specsync/archive/changes/2026-07-29-CHG-0006-v2-downgrade/state.json)"
expect_value "$v2_downgrade" archive_only false
expect_value "$v2_downgrade" legacy_archive_only false
expect_value "$v2_downgrade" archive_workflow_version 1
expect_value "$v2_downgrade" archive_parent_workflow_version 2
expect_value "$v2_downgrade" full true

active_edit="$(classify M .specsync/changes/CHG-0002-active-widget/state.json)"
expect_value "$active_edit" archive_only false
expect_value "$active_edit" review_only false

review_record="$(classify \
    A .specsync/changes/CHG-0002-active-widget/review.json \
    A .specsync/changes/CHG-0002-active-widget/review-attempts.json)"
expect_value "$review_record" review_only true
expect_value "$review_record" review_change_id CHG-0002-active-widget
expect_value "$review_record" full false

review_with_state="$(classify \
    A .specsync/changes/CHG-0002-active-widget/review.json \
    A .specsync/changes/CHG-0002-active-widget/review-attempts.json \
    M .specsync/changes/CHG-0002-active-widget/state.json)"
expect_value "$review_with_state" review_only false

modified_review="$(classify M .specsync/changes/CHG-0002-active-widget/review.json)"
expect_value "$modified_review" review_only false

appended_review="$(classify \
    M .specsync/changes/CHG-0002-active-widget/review.json \
    M .specsync/changes/CHG-0002-active-widget/review-attempts.json)"
expect_value "$appended_review" review_only true
expect_value "$appended_review" review_change_id CHG-0002-active-widget
expect_value "$appended_review" full false

review_fixture="$fixture/review-recovery"
review_change="$review_fixture/.specsync/changes/CHG-0007-review-recovery"
mkdir -p "$review_change" "$review_fixture/.github/scripts"
cp "$script_dir/lifecycle-validation-limits.json" "$review_fixture/.github/scripts/"
git -C "$review_fixture" init -q
git -C "$review_fixture" config user.email classifier@specsync.dev
git -C "$review_fixture" config user.name "SpecSync Classifier"
printf '%s\n' '{"id":"CHG-0007-review-recovery","state":"verifying","workflow_version":2}' \
    > "$review_change/state.json"
printf '%s\n' '{"passed":true}' > "$review_change/verification.json"
git -C "$review_fixture" add .
git -C "$review_fixture" commit -qm "implementation"
printf '%s\n' '{"verdict":"block"}' > "$review_change/review.json"
printf '%s\n' '{"schema_version":1,"reviews":[{"verdict":"block"}]}' \
    > "$review_change/review-attempts.json"
git -C "$review_fixture" add .
git -C "$review_fixture" commit -qm "blocking review"
block_commit="$(git -C "$review_fixture" rev-parse HEAD)"
printf '%s\n' '{"verdict":"pass"}' > "$review_change/review.json"
printf '%s\n' \
    '{"schema_version":1,"reviews":[{"verdict":"block"},{"verdict":"pass"}]}' \
    > "$review_change/review-attempts.json"
git -C "$review_fixture" add .
git -C "$review_fixture" commit -qm "passing review"
pass_commit="$(git -C "$review_fixture" rev-parse HEAD)"
block_to_pass="$(
    git -C "$review_fixture" diff --name-status -z "$block_commit" "$pass_commit" \
        | "$classifier" "$review_fixture" false name-status "$block_commit"
)"
expect_value "$block_to_pass" review_only true
expect_value "$block_to_pass" review_change_id CHG-0007-review-recovery
expect_value "$block_to_pass" full false

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

stale_review="{\"schema_version\":2,\"change_id\":\"CHG-0002-active-widget\",\"reviewer\":\"Independent reviewer\",\"provenance\":{\"schema_version\":1,\"provider\":\"github_actions_check\",\"required_check\":\"SpecSync scoped review\"},\"verdict\":\"pass\",\"implementation_commit\":\"$fixture_head\",\"contract_digest\":\"contract\",\"execution_digest\":\"stale\",\"workspace_digest\":\"current\",\"timestamp\":2}"
printf '%s\n' "$stale_review" \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/review.json"
printf '{"schema_version":1,"reviews":[%s]}\n' "$stale_review" \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/review-attempts.json"
stale_execution_review="$(classify M specs/parser/parser.spec.md)"
expect_value "$stale_execution_review" review_required true

self_review_json="{\"schema_version\":2,\"change_id\":\"CHG-0002-active-widget\",\"reviewer\":\"Scope owner\",\"provenance\":{\"schema_version\":1,\"provider\":\"github_actions_check\",\"required_check\":\"SpecSync scoped review\"},\"verdict\":\"pass\",\"implementation_commit\":\"$fixture_head\",\"contract_digest\":\"contract\",\"execution_digest\":\"execution\",\"workspace_digest\":\"current\",\"timestamp\":2}"
printf '%s\n' "$self_review_json" \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/review.json"
printf '{"schema_version":1,"reviews":[%s]}\n' "$self_review_json" \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/review-attempts.json"
self_review="$(classify M specs/parser/parser.spec.md)"
expect_value "$self_review" review_required true

current_review_json="{\"schema_version\":2,\"change_id\":\"CHG-0002-active-widget\",\"reviewer\":\"Independent reviewer\",\"provenance\":{\"schema_version\":1,\"provider\":\"github_actions_check\",\"required_check\":\"SpecSync scoped review\"},\"verdict\":\"pass\",\"implementation_commit\":\"$fixture_head\",\"contract_digest\":\"contract\",\"execution_digest\":\"execution\",\"workspace_digest\":\"current\",\"timestamp\":2}"
printf '%s\n' "$current_review_json" \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/review.json"
printf '{"schema_version":1,"reviews":[%s]}\n' "$current_review_json" \
    > "$fixture/.specsync/changes/CHG-0002-active-widget/review-attempts.json"
current_review="$(classify M specs/parser/parser.spec.md)"
expect_value "$current_review" review_required false

mkdir -p "$fixture/src"
printf '%s\n' 'temporary behavior' > "$fixture/src/parser.rs"
git -C "$fixture" add src/parser.rs
git -C "$fixture" commit -qm "change implementation after review"
git -C "$fixture" rm -q src/parser.rs
git -C "$fixture" commit -qm "revert implementation after review"
reverted_review="$(classify M specs/parser/parser.spec.md)"
expect_value "$reverted_review" review_required true
expect_value "$reverted_review" review_required_change_id CHG-0002-active-widget

# ── lane selection: a tip-only answer may narrow, never contradict (#626) ──
#
# `specsync change ship` always produces an archive commit last, so before this
# rule existed the tip-only classification overrode the whole-PR one on EVERY
# lifecycle pull request. PR #629 changed nine source files and merged with
# test, fmt, coverage, audit and spec-check all skipped, aggregate green.
select_lane="$script_dir/select-ci-lane.sh"

lane_full="$(mktemp)"; lane_tip="$(mktemp)"
trap 'rm -f "$lane_full" "$lane_tip"' EXIT

# whole PR touched product paths; tip is an archive move
printf 'src/main.rs\0' | "$classifier" "$fixture" >"$lane_full"
printf 'archive_only=true\nlegacy_archive_only=false\narchive_attempted=true\nreview_only=false\nreview_required=false\nfull=false\nsite=false\nvscode=false\n' >"$lane_tip"
selected="$("$select_lane" "$lane_full" "$lane_tip")"
expect_value "$selected" full true
expect_value "$selected" archive_only false

# whole PR is archive-only too; the tip answer may narrow
printf '.specsync/archive/changes/x/state.json\0' | "$classifier" "$fixture" \
    | sed 's/^full=true/full=false/' >"$lane_full"
selected_narrow="$("$select_lane" "$lane_full" "$lane_tip")"
expect_value "$selected_narrow" archive_only true

# no tip candidate at all: the whole-PR answer stands
printf 'src/main.rs\0' | "$classifier" "$fixture" >"$lane_full"
selected_none="$("$select_lane" "$lane_full" "")"
expect_value "$selected_none" full true

# The mandatory independent review must be required for a change whose ID carries no
# ordinal. Globbing `CHG-*` here meant review_candidates=0, review_required=false, and a PR
# that merged without the review while CI went green FASTER — the worst possible failure
# shape for a gate. Identity is read from state.json, so the name must not matter.
shape_fixture="$fixture/shape-independent"
mkdir -p "$shape_fixture/.specsync/changes/retire-the-widget" "$shape_fixture/.github/scripts"
cp "$script_dir/lifecycle-validation-limits.json" "$shape_fixture/.github/scripts/"
printf '%s\n' \
    '{"id":"retire-the-widget","state":"verifying"}' \
    > "$shape_fixture/.specsync/changes/retire-the-widget/state.json"
printf '%s\n' \
    '{"passed":true,"contract_digest":"contract","execution_digest":"execution","workspace_digest":"current"}' \
    > "$shape_fixture/.specsync/changes/retire-the-widget/verification.json"
printf '%s\n' \
    '{"approvals":[{"gate":"definition","actor":"Scope owner","timestamp":1,"digest":"scope"}]}' \
    > "$shape_fixture/.specsync/changes/retire-the-widget/approvals.json"
printf '%s\n' \
    '{"change_id":"retire-the-widget","contract_digest":"contract","workspace_digest":"stale"}' \
    > "$shape_fixture/.specsync/changes/retire-the-widget/review.json"
printf '%s\n' \
    '{"schema_version":1,"reviews":[{"change_id":"retire-the-widget","contract_digest":"contract","workspace_digest":"stale"}]}' \
    > "$shape_fixture/.specsync/changes/retire-the-widget/review-attempts.json"
git -C "$shape_fixture" init -q
git -C "$shape_fixture" config user.email classifier@specsync.dev
git -C "$shape_fixture" config user.name "SpecSync Classifier"
git -C "$shape_fixture" add .
git -C "$shape_fixture" commit -qm "slug-only identity"
shape_result="$(printf '%s\0' src/main.rs \
    | "$classifier" "$shape_fixture" false name-status)"
expect_value "$shape_result" review_required true
expect_value "$shape_result" review_required_change_id retire-the-widget

echo "classify-ci-paths tests passed"

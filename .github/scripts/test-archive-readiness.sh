#!/usr/bin/env bash
# The Archive check's three answers: one open change, several, and none.
set -euo pipefail

script="$(cd "$(dirname "$0")" && pwd)/archive-readiness.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

write_change() {
    local id="$1" state="$2" ship="$3"
    local dir="$tmp/.specsync/changes/$id"
    mkdir -p "$dir"
    printf '{"state":"%s"}\n' "$state" >"$dir/state.json"
    if [[ "$ship" == "yes" ]]; then
        printf '{}\n' >"$dir/review.json"
        printf '{}\n' >"$dir/verification.json"
    fi
}

assert_fail() {
    local expected="$1"
    shift
    local output status
    set +e
    output="$("$script" "$tmp" 2>&1)"
    status=$?
    set -e
    [[ "$status" -ne 0 ]] || { echo "expected failure, got: $output" >&2; exit 1; }
    [[ "$output" == "$expected" ]] || { echo "got: $output" >&2; echo "want: $expected" >&2; exit 1; }
}

assert_pass() {
    local output status
    set +e
    output="$("$script" "$tmp" 2>&1)"
    status=$?
    set -e
    [[ "$status" -eq 0 ]] || { echo "expected success, got ($status): $output" >&2; exit 1; }
    [[ "$output" == "All changes are archived. Ready to merge when the other checks are green." ]] || {
        echo "got: $output" >&2
        exit 1
    }
}

write_change draft-notes draft no
assert_fail "This pull request is not finished. \`draft-notes\` is still open. Run \`specsync change status draft-notes\`."

write_change zebra-feature verifying yes
assert_fail "This pull request is not finished. Archive \`zebra-feature\` first. Still open: \`zebra-feature\`, \`draft-notes\`. Run \`specsync change ship zebra-feature\`."

rm -rf "$tmp/.specsync"
assert_pass

echo "archive readiness ok"

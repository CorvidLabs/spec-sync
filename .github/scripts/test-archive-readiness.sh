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
    if [[ "$ship" == "yes" ]]; then
        printf '{"state":"%s","workflow_version":2}\n' "$state" >"$dir/state.json"
        printf '{"verdict":"pass","implementation_commit":"%s"}\n' "$head" >"$dir/review.json"
        printf '{}\n' >"$dir/verification.json"
    else
        printf '{"state":"%s"}\n' "$state" >"$dir/state.json"
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

git -C "$tmp" init -b main >/dev/null
git -C "$tmp" config user.email archive@specsync.dev
git -C "$tmp" config user.name "Archive check"
printf 'seed\n' >"$tmp/README.md"
git -C "$tmp" add README.md
git -C "$tmp" commit -m "seed" >/dev/null
head="$(git -C "$tmp" rev-parse HEAD)"

write_change draft-notes draft no
assert_fail "This pull request is not finished. \`draft-notes\` is still open. Run \`specsync change status draft-notes\`."

write_change zebra-feature verifying yes
assert_fail "This pull request is not finished. Archive \`zebra-feature\` first. Still open: \`zebra-feature\`, \`draft-notes\`. Run \`specsync change ship zebra-feature\`."

# A blocking review must not be told to ship.
printf '{"verdict":"block","implementation_commit":"%s"}\n' "$head" >"$tmp/.specsync/changes/zebra-feature/review.json"
assert_fail "This pull request is not finished. Archive \`zebra-feature\` first. Still open: \`zebra-feature\`, \`draft-notes\`. Run \`specsync change status zebra-feature\`."

# Accepted is closer to the archive than a verifying change.
printf '{"verdict":"pass","implementation_commit":"%s"}\n' "$head" >"$tmp/.specsync/changes/zebra-feature/review.json"
write_change almost-done accepted no
assert_fail "This pull request is not finished. Archive \`almost-done\` first. Still open: \`almost-done\`, \`zebra-feature\`, \`draft-notes\`. Run \`specsync change status almost-done\`."

# A state that says archived, left in the active tree, is still open.
rm -rf "$tmp/.specsync"
mkdir -p "$tmp/.specsync/changes/left-behind"
printf '{"state":"archived"}\n' >"$tmp/.specsync/changes/left-behind/state.json"
assert_fail "This pull request is not finished. \`left-behind\` is still open. Run \`specsync change status left-behind\`."

rm -rf "$tmp/.specsync"
assert_pass

echo "archive readiness ok"

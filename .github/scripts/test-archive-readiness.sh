#!/usr/bin/env bash
# The Archive check's three answers: one open change, several, and none,
# and never `ship` for a copy the remote default already archived (#902).
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

# A directory that holds a file but no state.json is damaged and still open.
# An empty deltas/ sibling is a husk and must not appear in the sentence.
rm -rf "$tmp/.specsync"
mkdir -p "$tmp/.specsync/changes/broken-notes"
printf '# context\n' >"$tmp/.specsync/changes/broken-notes/context.md"
mkdir -p "$tmp/.specsync/changes/left-husk/deltas"
assert_fail "This pull request is not finished. \`broken-notes\` is still open. Run \`specsync change status broken-notes\`."

# A regular file nested under deltas/ is the same damage, not a husk.
rm -rf "$tmp/.specsync"
mkdir -p "$tmp/.specsync/changes/nested-damage/deltas"
printf 'note\n' >"$tmp/.specsync/changes/nested-damage/deltas/note.md"
assert_fail "This pull request is not finished. \`nested-damage\` is still open. Run \`specsync change status nested-damage\`."

# A symlink is a file Git can track, so it is not a husk.
rm -rf "$tmp/.specsync"
mkdir -p "$tmp/.specsync/changes/linked-notes"
ln -s missing-target "$tmp/.specsync/changes/linked-notes/pointer"
assert_fail "This pull request is not finished. \`linked-notes\` is still open. Run \`specsync change status linked-notes\`."

# An empty deltas/ husk is not an open change.
rm -rf "$tmp/.specsync"
mkdir -p "$tmp/.specsync/changes/left-husk/deltas"
assert_pass

rm -rf "$tmp/.specsync"
assert_pass

# A copy the remote default already archived must not be told to ship, even with a
# passing review at HEAD and a verification record (#902). The upstream archive is
# added by a commit HEAD does not contain, the test `upstream_archive_of` makes.
seed="$head"
upstream="$(mktemp -d)"
trap 'rm -rf "$tmp" "$upstream"' EXIT
git init -q --bare -b main "$upstream/origin.git"
git -C "$tmp" checkout -q -b publish
mkdir -p "$tmp/.specsync/archive/changes/2026-11-06-stale-copy"
printf '{"id":"stale-copy","state":"archived","workflow_version":2}\n' \
    >"$tmp/.specsync/archive/changes/2026-11-06-stale-copy/state.json"
git -C "$tmp" add .specsync/archive
git -C "$tmp" commit -q -m "archive stale-copy upstream"
archived="$(git -C "$tmp" rev-parse HEAD)"
git -C "$tmp" remote add origin "$upstream/origin.git"
git -C "$tmp" push -q origin "$archived:refs/heads/main"
git -C "$tmp" fetch -q origin
git -C "$tmp" remote set-head origin main >/dev/null
git -C "$tmp" checkout -q main
write_change stale-copy verifying yes
assert_fail "This pull request is not finished. \`stale-copy\` is still open. Run \`specsync change status stale-copy\`."

# An archive HEAD already contains is the reopen path: still told to ship.
git -C "$tmp" merge -q --ff-only "$archived"
head="$(git -C "$tmp" rev-parse HEAD)"
printf '{"verdict":"pass","implementation_commit":"%s"}\n' "$head" >"$tmp/.specsync/changes/stale-copy/review.json"
assert_fail "This pull request is not finished. \`stale-copy\` is still open. Run \`specsync change ship stale-copy\`."

# With no remote default ref the script cannot tell, and keeps its classification.
git -C "$tmp" checkout -q -B main "$seed"
git -C "$tmp" remote remove origin
head="$seed"
printf '{"verdict":"pass","implementation_commit":"%s"}\n' "$head" >"$tmp/.specsync/changes/stale-copy/review.json"
assert_fail "This pull request is not finished. \`stale-copy\` is still open. Run \`specsync change ship stale-copy\`."

echo "archive readiness ok"

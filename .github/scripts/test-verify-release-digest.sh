#!/usr/bin/env bash
# The Action's independently pinned release digest (#476): with `expected-sha256` set, the archive
# must match the line for this platform, a platform with no line fails closed, and a malformed
# line is refused. Without the input nothing is checked here (the .sha256 sidecar still is).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
verify="$here/verify-release-digest.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

archive="$tmp/specsync-linux-x86_64.tar.gz"
printf 'release bytes\n' > "$archive"
digest="$(shasum -a 256 "$archive" | cut -d' ' -f1)"
other="$(printf '%064d' 0)"

expect_pass() {
    if ! bash "$verify" "$archive" "$1" "$2" > "$tmp/out" 2>&1; then
        echo "FAIL: expected pass ($3):"; cat "$tmp/out"; exit 1
    fi
}
expect_fail() {
    if bash "$verify" "$archive" "$1" "$2" > "$tmp/out" 2>&1; then
        echo "FAIL: expected refusal ($3):"; cat "$tmp/out"; exit 1
    fi
    grep -q "$4" "$tmp/out" || { echo "FAIL: refusal does not say '$4' ($3):"; cat "$tmp/out"; exit 1; }
}

expect_pass linux-x86_64 "" "no expected digest configured"
expect_pass linux-x86_64 "linux-x86_64=$digest" "matching digest"
expect_pass linux-x86_64 "$(printf 'macos-aarch64=%s\n  linux-x86_64=%s  \n' "$other" "$digest")" "one line per platform, padded"
expect_fail linux-x86_64 "linux-x86_64=$other" "mismatched digest" "does not match"
expect_fail linux-x86_64 "macos-aarch64=$digest" "no line for this platform" "no checksum for linux-x86_64"
expect_fail linux-x86_64 "linux-x86_64=nothex" "malformed line" "not a 64-character"
expect_fail linux-x86_64 "$(printf 'linux-x86_64=%s\nlinux-x86_64=%s\n' "$digest" "$other")" "two lines for one platform" "more than once"
echo "verify-release-digest tests passed"

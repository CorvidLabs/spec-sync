#!/usr/bin/env bash
# Check a downloaded release archive against the digest a consumer pinned for its platform
# (#476). Usage: verify-release-digest.sh <archive> <platform> <expected-sha256 lines>
#
# The expected input holds one `<platform>=<sha256>` line per platform, for example
# `linux-x86_64=<64 hex>`. Empty means no digest was pinned and nothing is checked here; the
# release's own `.sha256` sidecar is still checked by the caller. Otherwise the line for this
# platform must exist exactly once and match, or the archive is refused before extraction.
set -euo pipefail

archive="$1"
platform="$2"
expected="$3"

if [[ -z "${expected//[[:space:]]/}" ]]; then
    exit 0
fi

pinned=""
count=0
while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line#"${line%%[![:space:]]*}"}"
    line="${line%"${line##*[![:space:]]}"}"
    [[ -z "$line" ]] && continue
    name="${line%%=*}"
    value="${line#*=}"
    if [[ "$name" == "$line" || -z "$name" ]]; then
        echo "::error::expected-sha256 line '$line' is not '<platform>=<sha256>'"
        exit 1
    fi
    if [[ ! "$value" =~ ^[0-9a-fA-F]{64}$ ]]; then
        echo "::error::expected-sha256 for $name is not a 64-character hex SHA-256 digest"
        exit 1
    fi
    if [[ "$name" == "$platform" ]]; then
        pinned="$value"
        count=$((count + 1))
    fi
done <<< "$expected"

if [[ "$count" -eq 0 ]]; then
    echo "::error::expected-sha256 has no checksum for $platform; add a '$platform=<sha256>' line"
    exit 1
fi
if [[ "$count" -gt 1 ]]; then
    echo "::error::expected-sha256 names $platform more than once"
    exit 1
fi

actual="$(shasum -a 256 "$archive" | cut -d' ' -f1)"
if [[ "$(printf '%s' "$actual" | tr 'A-F' 'a-f')" != "$(printf '%s' "$pinned" | tr 'A-F' 'a-f')" ]]; then
    echo "::error::the downloaded $platform archive's SHA-256 ($actual) does not match expected-sha256 ($pinned); refusing to install it"
    exit 1
fi
echo "::notice::Pinned SHA-256 verified for $platform"

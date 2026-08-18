#!/usr/bin/env bash
# Choose between the whole-PR classification and a narrower tip-only one.
#
# A lifecycle archive commit legitimately needs no product lane, and routing it
# through one costs a full suite for a diff that moves files inside
# `.specsync/archive/`. That is why a tip-only classification may narrow the
# lane at all.
#
# It may only NARROW, never contradict. `specsync change ship` always produces
# an archive commit last, so before this guard existed the tip-only
# classification overrode the whole-PR one on EVERY lifecycle pull request —
# and a pull request that changed nine source files merged with `test`, `fmt`,
# `coverage`, `audit` and `spec-check` all skipped and the aggregate green
# (CorvidLabs/spec-sync#626).
#
# Usage: select-ci-lane.sh <full-classification-file> <tip-classification-file>
# Emits the winning classification on stdout.
set -euo pipefail

full_output="$(cat "$1")"
tip_output="${2:+$(cat "$2")}"

# No tip candidate: the whole-PR answer stands.
if [[ -z "$tip_output" ]]; then
    printf '%s\n' "$full_output"
    exit 0
fi

# The tip must actually be one of the narrow kinds to be eligible.
if ! grep -Eq '^(archive_only|legacy_archive_only|review_only)=true$' <<<"$tip_output"; then
    printf '%s\n' "$full_output"
    exit 0
fi

# The decisive rule: if the whole pull request selected the product lane, an
# archive-shaped tip does not deselect it. An archive tip on a pull request
# that also touched product paths is not an archive-only pull request.
if grep -Eq '^full=true$' <<<"$full_output"; then
    printf '%s\n' "$full_output"
    exit 0
fi

printf '%s\n' "$tip_output"

#!/usr/bin/env bash

set -euo pipefail

root="${1:-.}"
force_full="${2:-false}"

archive_candidate=true
archive_seen=false
changed_seen=false
full=false
site=false
vscode=false

if [[ "$force_full" == "true" ]]; then
    archive_candidate=false
    full=true
fi

while IFS= read -r -d '' path; do
    changed_seen=true

    case "$path" in
        .specsync/archive/changes/*)
            archive_seen=true
            ;;
        .specsync/changes/*)
            if [[ -e "$root/$path" || -L "$root/$path" ]]; then
                archive_candidate=false
            fi
            ;;
        *)
            archive_candidate=false
            ;;
    esac

    case "$path" in
        src/*|tests/*|Cargo.toml|Cargo.lock|rust-toolchain.toml|action.yml|examples/*|fledge.toml|.github/workflows/*|.github/scripts/*)
            full=true
            ;;
        site/*)
            site=true
            ;;
        vscode-extension/*)
            vscode=true
            ;;
        specs/*|.specsync/*)
            ;;
        *)
            full=true
            ;;
    esac
done

archive_only=false
if [[ "$changed_seen" == "true" && "$archive_candidate" == "true" && "$archive_seen" == "true" ]]; then
    archive_only=true
    full=false
    site=false
    vscode=false
fi

printf 'archive_only=%s\n' "$archive_only"
printf 'full=%s\n' "$full"
printf 'site=%s\n' "$site"
printf 'vscode=%s\n' "$vscode"

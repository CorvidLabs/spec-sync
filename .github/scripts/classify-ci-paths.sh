#!/usr/bin/env bash

set -euo pipefail

root="${1:-.}"
force_full="${2:-false}"
input_format="${3:-names}"

archive_candidate=true
archive_attempted=false
archive_seen=false
active_seen=false
changed_seen=false
full=false
site=false
vscode=false
archive_change_id=""
archive_dir=""
review_candidate=true
review_seen=false
review_change_id=""

if [[ "$force_full" == "true" ]]; then
    archive_candidate=false
    review_candidate=false
    full=true
fi

classify_path() {
    local path="$1"

    changed_seen=true
    case "$path" in
        .specsync/archive/changes/*)
            archive_attempted=true
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
}

record_active_path() {
    local path="$1" remainder change_id

    case "$path" in
        .specsync/changes/*/*)
            remainder="${path#.specsync/changes/}"
            change_id="${remainder%%/*}"
            ;;
        *)
            archive_candidate=false
            return
            ;;
    esac

    if [[ -z "$change_id" || "$change_id" == "$remainder" ]]; then
        archive_candidate=false
        return
    fi
    if [[ -n "$archive_change_id" && "$archive_change_id" != "$change_id" ]]; then
        archive_candidate=false
        return
    fi

    archive_change_id="$change_id"
    active_seen=true
}

record_archive_path() {
    local path="$1" remainder candidate_dir dated_id change_id

    case "$path" in
        .specsync/archive/changes/*/*)
            remainder="${path#.specsync/archive/changes/}"
            candidate_dir="${remainder%%/*}"
            ;;
        *)
            archive_candidate=false
            return
            ;;
    esac

    if [[ ! "$candidate_dir" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}-(CHG-[0-9]{4,}-.+)$ ]]; then
        archive_candidate=false
        return
    fi
    dated_id="${BASH_REMATCH[0]}"
    change_id="${BASH_REMATCH[1]}"
    if [[ -n "$archive_change_id" && "$archive_change_id" != "$change_id" ]]; then
        archive_candidate=false
        return
    fi
    if [[ -n "$archive_dir" && "$archive_dir" != "$dated_id" ]]; then
        archive_candidate=false
        return
    fi

    archive_change_id="$change_id"
    archive_dir="$dated_id"
    archive_seen=true
}

record_review_change() {
    local status="$1" path="$2"

    if [[ "$review_candidate" != "true" || "$review_seen" == "true" || "$status" != "A" ]]; then
        review_candidate=false
        return
    fi
    if [[ "$path" =~ ^\.specsync/changes/(CHG-[0-9]{4,}-.+)/review\.json$ ]]; then
        review_change_id="${BASH_REMATCH[1]}"
        review_seen=true
    else
        review_candidate=false
    fi
}

if [[ "$input_format" == "name-status" ]]; then
    while IFS= read -r -d '' status; do
        changed_seen=true

        if ! IFS= read -r -d '' first_path; then
            archive_candidate=false
            break
        fi

        record_review_change "$status" "$first_path"
        case "$status" in
            R[0-9]*|C[0-9]*)
                if ! IFS= read -r -d '' second_path; then
                    archive_candidate=false
                    break
                fi
                classify_path "$first_path"
                classify_path "$second_path"
                if [[ "$status" == R* ]]; then
                    record_active_path "$first_path"
                    record_archive_path "$second_path"
                else
                    archive_candidate=false
                fi
                ;;
            D)
                classify_path "$first_path"
                record_active_path "$first_path"
                ;;
            A)
                classify_path "$first_path"
                record_archive_path "$first_path"
                ;;
            *)
                classify_path "$first_path"
                archive_candidate=false
                ;;
        esac
    done
else
    # A name-only stream remains useful for selecting the normal product
    # lanes, but it cannot prove that a finalization commit moved one exact
    # active change into its matching archive. Never grant archive-only
    # treatment without Git status information.
    archive_candidate=false
    review_candidate=false
    while IFS= read -r -d '' path; do
        classify_path "$path"
    done
fi

archive_only=false
if [[ "$changed_seen" == "true" \
    && "$archive_candidate" == "true" \
    && "$active_seen" == "true" \
    && "$archive_seen" == "true" \
    && -n "$archive_change_id" \
    && -n "$archive_dir" \
    && ! -e "$root/.specsync/changes/$archive_change_id" \
    && -d "$root/.specsync/archive/changes/$archive_dir" ]]; then
    archive_only=true
    full=false
    site=false
    vscode=false
fi
if [[ "$archive_only" != "true" && "$archive_attempted" == "true" ]]; then
    # Bulk migration/cleanup archives and malformed finalization candidates use
    # the full lane. Only a positively proven one-change child may bypass it.
    full=true
fi

review_only=false
if [[ "$changed_seen" == "true" \
    && "$review_candidate" == "true" \
    && "$review_seen" == "true" \
    && -n "$review_change_id" \
    && -f "$root/.specsync/changes/$review_change_id/review.json" ]]; then
    review_only=true
    full=false
    site=false
    vscode=false
fi

review_required=false
review_required_change_id=""
if [[ "$archive_only" != "true" && "$review_only" != "true" ]] && command -v jq >/dev/null 2>&1; then
    review_candidates=0
    for state_path in "$root"/.specsync/changes/CHG-*/state.json; do
        [[ -f "$state_path" ]] || continue
        change_dir="${state_path%/state.json}"
        verification_path="$change_dir/verification.json"
        review_path="$change_dir/review.json"
        [[ -f "$verification_path" ]] || continue

        state_id="$(jq -r 'if .state == "verifying" then .id // "" else "" end' "$state_path" 2>/dev/null || true)"
        verification_passed="$(jq -r '.passed == true' "$verification_path" 2>/dev/null || true)"
        [[ -n "$state_id" && "$verification_passed" == "true" ]] || continue

        current_review=false
        if [[ -f "$review_path" ]]; then
            current_review="$(
                jq -n \
                    --slurpfile review "$review_path" \
                    --slurpfile verification "$verification_path" \
                    --arg state_id "$state_id" \
                    '
                      ($review[0].change_id == $state_id)
                      and ($review[0].contract_digest == $verification[0].contract_digest)
                      and ($review[0].workspace_digest == $verification[0].workspace_digest)
                    ' 2>/dev/null || true
            )"
        fi
        if [[ "$current_review" != "true" ]]; then
            review_candidates=$((review_candidates + 1))
            review_required_change_id="$state_id"
        fi
    done
    if [[ "$review_candidates" == "1" ]]; then
        review_required=true
    else
        review_required_change_id=""
    fi
fi

printf 'archive_only=%s\n' "$archive_only"
printf 'archive_attempted=%s\n' "$archive_attempted"
printf 'review_only=%s\n' "$review_only"
printf 'review_required=%s\n' "$review_required"
printf 'full=%s\n' "$full"
printf 'site=%s\n' "$site"
printf 'vscode=%s\n' "$vscode"
printf 'archive_change_id=%s\n' "$archive_change_id"
printf 'archive_dir=%s\n' "$archive_dir"
printf 'review_change_id=%s\n' "$review_change_id"
printf 'review_required_change_id=%s\n' "$review_required_change_id"

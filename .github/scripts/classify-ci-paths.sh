#!/usr/bin/env bash

set -euo pipefail

root="${1:-.}"
force_full="${2:-false}"
input_format="${3:-names}"
parent_ref="${4:-}"
limits_file="$root/.github/scripts/lifecycle-validation-limits.json"

if ! command -v jq >/dev/null 2>&1 || [[ ! -f "$limits_file" ]]; then
    echo "lifecycle CI classification requires jq and the shared limits file" >&2
    exit 1
fi
jq -e '
  [
    .git_max_output_bytes,
    .git_timeout_seconds,
    .scoped_review_max_descendants,
    .scoped_review_max_parents
  ]
  | all(type == "number" and . > 0 and floor == .)
' "$limits_file" >/dev/null

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
archive_workflow_version=""
archive_parent_workflow_version=""
review_candidate=true
review_seen=false
review_attempts_seen=false
review_change_id=""

review_history_is_current() {
    local reviewed_commit="$1"
    local active_dir="$2"
    python3 - "$root" "$reviewed_commit" "$active_dir" "$limits_file" <<'PY'
import json
import os
import selectors
import subprocess
import sys
import time

root, reviewed_commit, active_dir, limits_path = sys.argv[1:]
with open(limits_path, encoding="utf-8") as handle:
    limits = json.load(handle)
output_limit = int(limits["git_max_output_bytes"])
timeout = int(limits["git_timeout_seconds"])
descendant_limit = int(limits["scoped_review_max_descendants"])
parent_limit = int(limits["scoped_review_max_parents"])


def git_bytes(*args: str) -> bytes:
    process = subprocess.Popen(
        ["git", "-C", root, *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if process.stdout is None or process.stderr is None:
        process.kill()
        raise RuntimeError("missing Git pipes")
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ, "stdout")
    selector.register(process.stderr, selectors.EVENT_READ, "stderr")
    stdout = bytearray()
    total = 0
    deadline = time.monotonic() + timeout
    try:
        while selector.get_map():
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError
            events = selector.select(timeout=remaining)
            for key, _ in events:
                chunk = os.read(key.fileobj.fileno(), 64 * 1024)
                if not chunk:
                    selector.unregister(key.fileobj)
                    continue
                total += len(chunk)
                if total > output_limit:
                    raise OverflowError
                if key.data == "stdout":
                    stdout.extend(chunk)
        if process.wait(timeout=5) != 0:
            raise RuntimeError("Git query failed")
        return bytes(stdout)
    finally:
        selector.close()
        if process.poll() is None:
            process.kill()
            process.wait()


def git_text(*args: str) -> str:
    return git_bytes(*args).decode().strip()


def git_status(*args: str) -> int:
    return subprocess.run(
        ["git", "-C", root, *args],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
        timeout=timeout,
    ).returncode


try:
    head = git_text("rev-parse", "--verify", "HEAD^{commit}")
    if git_text("rev-parse", "--verify", f"{reviewed_commit}^{{commit}}") != reviewed_commit:
        raise RuntimeError("noncanonical review commit")
    if git_status("merge-base", "--is-ancestor", reviewed_commit, head) != 0:
        raise RuntimeError("review commit is not an ancestor")
    descendants = git_text(
        "rev-list",
        "--reverse",
        f"--max-count={descendant_limit + 1}",
        f"{reviewed_commit}..{head}",
    ).splitlines()
    if len(descendants) > descendant_limit:
        raise RuntimeError("review history exceeds descendant limit")
    exclusions = [
        f":(top,exclude,literal){active_dir}/{name}"
        for name in (
            "review.json",
            "review-attempts.json",
            "state.json",
            "verification.json",
            "verification-attempts.json",
        )
    ]
    for commit in descendants:
        fields = git_text("rev-list", "--parents", "-n", "1", commit).split()
        if not fields or fields[0] != commit or not 1 <= len(fields) - 1 <= parent_limit:
            raise RuntimeError("review history has invalid parents")
        for parent in fields[1:]:
            if git_status(
                "diff",
                "--quiet",
                "--no-renames",
                parent,
                commit,
                "--",
                ".",
                *exclusions,
            ) != 0:
                raise RuntimeError("implementation changed after review")
except (
    json.JSONDecodeError,
    OSError,
    OverflowError,
    RuntimeError,
    subprocess.SubprocessError,
    TimeoutError,
    UnicodeDecodeError,
    ValueError,
):
    raise SystemExit(1)
PY
}

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

    # Identity comes from state.json, never from the shape of the directory name. The
    # `YYYY-MM-DD-CHG-NNNN-slug` regex that used to sit here made the archive fast lane
    # silently unavailable for any other naming shape — and it fails OPEN in the sibling
    # review check below, which is why both moved to state.json in one change.
    #
    # No identity means no fast lane: the full product matrix runs. That is the safe
    # direction, and it is what happens when jq is missing or the state is unreadable.
    dated_id="$candidate_dir"
    archive_state_path="$root/.specsync/archive/changes/$candidate_dir/state.json"
    change_id=""
    if [[ -f "$archive_state_path" ]] && command -v jq >/dev/null 2>&1; then
        change_id="$(jq -r '.id // ""' "$archive_state_path" 2>/dev/null || true)"
    fi
    if [[ -z "$change_id" ]]; then
        archive_candidate=false
        return
    fi
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
    local candidate_id=""

    if [[ "$review_candidate" != "true" \
        || ( "$status" != "A" && "$status" != "M" ) ]]; then
        review_candidate=false
        return
    fi
    if [[ "$path" =~ ^\.specsync/changes/([^/]+)/review\.json$ ]]; then
        candidate_id="${BASH_REMATCH[1]}"
        if [[ "$review_seen" == "true" ]]; then
            review_candidate=false
            return
        fi
        review_seen=true
    elif [[ "$path" =~ ^\.specsync/changes/([^/]+)/review-attempts\.json$ ]]; then
        candidate_id="${BASH_REMATCH[1]}"
        if [[ "$review_attempts_seen" == "true" ]]; then
            review_candidate=false
            return
        fi
        review_attempts_seen=true
    else
        review_candidate=false
        return
    fi
    if [[ -n "$review_change_id" && "$review_change_id" != "$candidate_id" ]]; then
        review_candidate=false
        return
    fi
    review_change_id="$candidate_id"
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

archive_shape=false
if [[ "$changed_seen" == "true" \
    && "$archive_candidate" == "true" \
    && "$active_seen" == "true" \
    && "$archive_seen" == "true" \
    && -n "$archive_change_id" \
    && -n "$archive_dir" \
    && ! -e "$root/.specsync/changes/$archive_change_id" \
    && -d "$root/.specsync/archive/changes/$archive_dir" ]]; then
    archive_shape=true
fi

archive_only=false
legacy_archive_only=false
if [[ "$archive_shape" == "true" ]] && command -v jq >/dev/null 2>&1; then
    archive_state="$root/.specsync/archive/changes/$archive_dir/state.json"
    if [[ -f "$archive_state" ]]; then
        archive_workflow_version="$(
            jq -er '
              (.workflow_version // 1) as $version
              | if ($version == 1 or $version == 2)
                then ($version | tostring)
                else error("unsupported workflow version")
                end
            ' "$archive_state" 2>/dev/null || true
        )"
    fi
fi

if [[ "$archive_workflow_version" == "2" ]]; then
    archive_only=true
    full=false
    site=false
    vscode=false
elif [[ "$archive_workflow_version" == "1" ]]; then
    if [[ -n "$parent_ref" ]]; then
        archive_parent_workflow_version="$(
            git -C "$root" show \
                "${parent_ref}:.specsync/changes/$archive_change_id/state.json" \
                2>/dev/null \
                | jq -er '
                    (.workflow_version // 1) as $version
                    | if ($version == 1 or $version == 2)
                      then ($version | tostring)
                      else error("unsupported workflow version")
                      end
                  ' 2>/dev/null || true
        )"
    fi
    if [[ "$archive_parent_workflow_version" == "1" ]]; then
        # Legacy workflow-v1 archives predate same-PR finalization evidence.
        # Authenticate the legacy version against the active parent so a v2
        # change cannot downgrade itself while moving into the archive.
        legacy_archive_only=true
    fi
    full=true
fi
if [[ "$archive_only" != "true" \
    && "$legacy_archive_only" != "true" \
    && "$archive_attempted" == "true" ]]; then
    # Bulk migration/cleanup archives and malformed finalization candidates use
    # the full lane. Only a positively proven one-change child may bypass it.
    full=true
fi

review_only=false
if [[ "$changed_seen" == "true" \
    && "$review_candidate" == "true" \
    && "$review_seen" == "true" \
    && "$review_attempts_seen" == "true" \
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
    # Globbing `CHG-*` here gated the one mandatory human review on a naming convention: any
    # identity shape the glob missed produced review_candidates=0, review_required=false, and a
    # PR that merged without the review while CI went green faster. The loop already reads `.id`
    # from state.json below — the glob was the only thing that did not.
    for state_path in "$root"/.specsync/changes/*/state.json; do
        [[ -f "$state_path" ]] || continue
        change_dir="${state_path%/state.json}"
        verification_path="$change_dir/verification.json"
        review_path="$change_dir/review.json"
        review_attempts_path="$change_dir/review-attempts.json"
        [[ -f "$verification_path" ]] || continue

        state_id="$(jq -r 'if .state == "verifying" then .id // "" else "" end' "$state_path" 2>/dev/null || true)"
        verification_passed="$(jq -r '.passed == true' "$verification_path" 2>/dev/null || true)"
        [[ -n "$state_id" && "$verification_passed" == "true" ]] || continue

        current_review=false
        approvals_path="$change_dir/approvals.json"
        if [[ -f "$review_path" && -f "$review_attempts_path" && -f "$approvals_path" ]]; then
            current_review="$(
                jq -n \
                    --slurpfile review "$review_path" \
                    --slurpfile attempts "$review_attempts_path" \
                    --slurpfile verification "$verification_path" \
                    --slurpfile approvals "$approvals_path" \
                    --arg state_id "$state_id" \
                    '
                      [
                        $approvals[0].approvals[]
                        | select(.gate == "definition")
                        | (.actor // "" | tostring)
                      ] as $scope_approvers
                      | ($scope_approvers | last // "") as $scope_approver
                      | ($review[0].schema_version == 2)
                      and ($attempts[0].schema_version == 1)
                      and (($attempts[0].reviews | last) == $review[0])
                      and ($review[0].change_id == $state_id)
                      and ($review[0].provenance == {
                        "schema_version": 1,
                        "provider": "github_actions_check",
                        "required_check": "SpecSync scoped review"
                      })
                      and ($review[0].verdict == "pass")
                      and (($review[0].reviewer // "" | tostring)
                        | test("^[A-Za-z0-9 ._:@/-]{1,128}$"))
                      and (($review[0].reviewer | ascii_downcase) != ($scope_approver | ascii_downcase))
                      and (($review[0].implementation_commit // "") | test("^[0-9a-f]{40,64}$"))
                      and ($review[0].contract_digest == $verification[0].contract_digest)
                      and ($review[0].execution_digest == $verification[0].execution_digest)
                      and ($review[0].workspace_digest == $verification[0].workspace_digest)
                    ' 2>/dev/null || true
            )"
            if [[ "$current_review" == "true" ]]; then
                reviewed_commit="$(jq -r '.implementation_commit // ""' "$review_path")"
                if ! review_history_is_current "$reviewed_commit" \
                    "${change_dir#"$root"/}"; then
                    current_review=false
                fi
            fi
        fi
        if [[ "$current_review" != "true" ]]; then
            review_candidates=$((review_candidates + 1))
            # Prefer the first unreviewed verifying change so multi-change PRs
            # can run the SpecSync scoped review check and record reviews one
            # at a time instead of deadlocking when candidates != 1.
            if [[ -z "$review_required_change_id" ]]; then
                review_required_change_id="$state_id"
            fi
        fi
    done
    # Any unreviewed verifying change needs the scoped-review CI check. Requiring
    # exactly one candidate left dual-CHG PRs stuck (check skipped → no review
    # provenance → cannot finalize either change).
    if [[ "$review_candidates" -ge 1 ]]; then
        review_required=true
    else
        review_required_change_id=""
    fi
fi

printf 'archive_only=%s\n' "$archive_only"
printf 'legacy_archive_only=%s\n' "$legacy_archive_only"
printf 'archive_attempted=%s\n' "$archive_attempted"
printf 'review_only=%s\n' "$review_only"
printf 'review_required=%s\n' "$review_required"
printf 'full=%s\n' "$full"
printf 'site=%s\n' "$site"
printf 'vscode=%s\n' "$vscode"
printf 'archive_change_id=%s\n' "$archive_change_id"
printf 'archive_dir=%s\n' "$archive_dir"
printf 'archive_workflow_version=%s\n' "$archive_workflow_version"
printf 'archive_parent_workflow_version=%s\n' "$archive_parent_workflow_version"
printf 'review_change_id=%s\n' "$review_change_id"
printf 'review_required_change_id=%s\n' "$review_required_change_id"

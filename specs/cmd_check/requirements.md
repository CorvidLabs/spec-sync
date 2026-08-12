---
spec: cmd_check.spec.md
---

## User Stories

- As a developer, I want `specsync check` to validate every spec against its source so that drift is caught before it ships
- As a developer on a large repo, I want unchanged specs skipped via a hash cache so that repeated checks stay fast, and `--force`/`--no-cache` to re-validate everything when I need it
- As a CI operator, I want exit codes driven by enforcement mode, `--strict`, and `--require-coverage` so that pipeline pass/fail is predictable
- As a developer, I want `--fix` to make safe deterministic markdown repairs while leaving contract judgment to me or my coding agent
- As a cautious developer, I want `--dry-run` to preview `--fix` and `--backup` to snapshot specs before they are rewritten so that I never lose work
- As a maintainer, I want `--stale [N]` to flag specs that are N+ commits behind their source files so that I can spot quietly-rotting docs
- As a tool integrator, I want `--format json/markdown/github` so that results render in dashboards, PR bodies, or Actions logs

## Acceptance Criteria

- Validates discovered specs, applying `--exclude-status`/`--only-status` and positional `[SPEC...]` filters
- A hash cache (`.specsync/hashes.json`) skips unchanged specs unless `--force`/`--no-cache`, `--strict`, or explicit spec filters are given; the skipped count is reported in text mode
- Requirements drift remains visible as validation guidance for humans and coding agents
- `--fix` renames near-miss headers and appends undocumented exports with language-aware skeleton rows; it performs no inference or command execution
- `--backup` copies specs to `.specsync/backup-fix/` before any `--fix` write, aborting on any copy/dir failure to avoid data loss
- `--dry-run` previews `--fix` without writing; `--dry-run` without `--fix` prints a warning that it has no effect
- `--stale [N]` (default N=5) runs only inside a git repo, using `git_last_commit_hash` + `git_commits_since` to count how many commits each source file has advanced past the spec's last commit, flagging specs ≥ N behind
- `--create-issues` creates one GitHub issue per spec with errors (only when `total_errors > 0`)
- The hash cache is updated and saved only when `total_errors == 0`
- JSON output is a single object with `passed`, `errors`, `warnings`, `stale`, and `specs_checked`
- Exit code comes from `compute_exit_code`/`exit_with_status` (Warn/EnforceNew/Strict + require-coverage)
- When `.specsync/sdd.json` enables SDD, unified check first validates change coverage, approvals, semantic conflicts, and code against the effective canonical-plus-approved-delta contract.

## Constraints

- Must not panic on expected error conditions — print and exit
- `--fix` only modifies spec markdown files, never source code
- `--fix` is deterministic and local regardless of inference-related environment variables
- `--stale` is a no-op outside a git repo (no crash); specs without a `files:` list are skipped
- Git staleness uses `git_commits_since` (single rev-list per file) — the earlier per-pair `git_commits_between` N+1 walk was removed

## Out of Scope

- Modifying or generating source code
- Defining the CLI grammar (lives in `src/cli.rs`)
- The scoring algorithm itself (lives in the scoring module; check only renders `--explain` breakdowns)
- Non-git staleness heuristics (filesystem mtimes, etc.)

### REQ-cmd-check-001

The `cmd_check` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-cmd-check-002

The check and fix pipeline SHALL remain deterministic and local.

Acceptance Criteria
- `--fix` performs deterministic markdown repairs and never invokes an embedded model or shell AI command.
- Requirements drift remains visible as validation guidance for a coding agent to resolve.
- Existing cache, enforcement, lifecycle, output-format, backup, and dry-run behavior remains intact.

### REQ-cmd-check-003

The primary check command SHALL consume one fallible schema snapshot and report warning
suppression truthfully in every supported output.

Acceptance Criteria

- Schema replay, table identity, pattern additions, and column validation derive from one immutable
  snapshot per validation invocation.
- Missing, unreadable, malformed, or vacuous configured schema input is an explicit finding and
  cannot become an empty successful comparison.
- Text, JSON, Markdown, and GitHub output distinguish emitted warnings from deterministic
  `suppressed_warnings` details.
- Strict exit behavior counts unsuppressed findings only and preserves existing cache and coverage
  semantics.

### REQ-cmd-check-004

The primary check command SHALL treat SDD lifecycle state as information and SHALL NOT
derive its exit status from it.

Acceptance Criteria

- The number of active changes is reported without affecting exit status in any supported
  output format.
- Workspace files that cannot be parsed, or that record an illegal state, produce an explicit
  shape warning rather than a gate failure.
- Exit status derives solely from spec validation results, the effective enforcement mode,
  `--strict`, and `--require-coverage`.
- Lifecycle gating remains reachable through the `change` verbs and `specsync change audit`,
  whose behavior is unchanged.


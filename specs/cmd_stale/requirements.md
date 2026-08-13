---
spec: cmd_stale.spec.md
---

## User Stories

- As a developer, I want to know which specs have drifted from their source files so I can update them
- As a CI operator, I want `specsync stale` to fail the build (non-zero exit) when drift is detected so drift is caught early
- As a maintainer, I want to scope the scan with `--only-status`/`--exclude-status` so I can ignore draft or archived specs

## Acceptance Criteria

- `specsync stale` lists specs whose source files have changed since the spec was last committed, sorted most-stale-first
- A spec is stale when any of its source files has `>= threshold` commits since the spec's last commit (default threshold: 5)
- Output reports per-spec commit count and the list of drifted source files (each with its own commit count)
- Honors the global `--exclude-status` / `--only-status` filters when selecting which specs to scan
- Supports `text`/`table`/`csv` (human), `json` (machine), and `markdown`/`github` output formats
- Exit code is 1 when any stale specs are detected, 0 when all are fresh (for CI usage)
- Requires a git repository: errors and exits 1 when `is_git_repo` returns false (JSON mode emits an error object instead of stderr text)

## Constraints

- Must not panic on expected error conditions — print and exit
- Must work with the project's Clap-based CLI argument parsing
- Git operations must handle missing git repos gracefully (non-git directories error cleanly rather than crashing)
- Staleness uses `git_commits_since` with one precomputed spec commit hash per spec (no N+1 `git log` calls)

## Out of Scope

- Auto-updating or regenerating stale specs (reporting only)
- File modification-time heuristics (git history is the sole source of truth)
- Combining staleness with coverage/validation status (that is the `report` command)

### REQ-cmd-stale-001

The `cmd_stale` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-cmd-stale-002

Staleness detection SHALL refuse to report a verdict when there is no history to derive one
from, and SHALL name which precondition is missing.

Acceptance Criteria
- A repository with no commits is refused and exits non-zero, rather than reporting specs up to date.
- A path that is not a repository continues to be refused and exits non-zero.
- The two causes are reported distinctly in both text and machine-readable output.
- A repository with at least one commit reports staleness normally and exits zero.

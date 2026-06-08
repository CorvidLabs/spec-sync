---
spec: cmd_check.spec.md
---

## User Stories

- As a developer, I want `specsync check` to validate every spec against its source so that drift is caught before it ships
- As a developer on a large repo, I want unchanged specs skipped via a hash cache so that repeated checks stay fast, and `--force`/`--no-cache` to re-validate everything when I need it
- As a CI operator, I want exit codes driven by enforcement mode, `--strict`, and `--require-coverage` so that pipeline pass/fail is predictable
- As a developer, I want `--fix` to add undocumented exports, fix near-miss headers, and (when requirements drifted) AI-regenerate the spec so that keeping specs current is low-friction
- As a cautious developer, I want `--dry-run` to preview `--fix` and `--backup` to snapshot specs before they are rewritten so that I never lose work
- As a maintainer, I want `--stale [N]` to flag specs that are N+ commits behind their source files so that I can spot quietly-rotting docs
- As a tool integrator, I want `--format json/markdown/github` so that results render in dashboards, PR bodies, or Actions logs

## Acceptance Criteria

- Validates discovered specs, applying `--exclude-status`/`--only-status` and positional `[SPEC...]` filters
- A hash cache (`.specsync/hashes.json`) skips unchanged specs unless `--force`/`--no-cache`, `--strict`, or explicit spec filters are given; the skipped count is reported in text mode
- Requirements-drift is detected per spec; in an interactive TTY (no `--fix`) the user is prompted to re-validate, and with `--fix` drifted specs are AI-regenerated
- `--fix` runs in passes: rename near-miss `## Required` and `### Export` headers, append undocumented exports to the Public API table (with language-aware skeleton rows), then AI-regenerate requirements-drifted specs
- `--backup` copies specs to `.specsync/backup-fix/` before any `--fix` write, aborting on any copy/dir failure to avoid data loss
- `--dry-run` previews `--fix` without writing; `--dry-run` without `--fix` prints a warning that it has no effect
- `--stale [N]` (default N=5) runs only inside a git repo, using `git_last_commit_hash` + `git_commits_since` to count how many commits each source file has advanced past the spec's last commit, flagging specs ≥ N behind
- `--create-issues` creates one GitHub issue per spec with errors (only when `total_errors > 0`)
- The hash cache is updated and saved only when `total_errors == 0`
- JSON output is a single object with `passed`, `errors`, `warnings`, `stale`, and `specs_checked`
- Exit code comes from `compute_exit_code`/`exit_with_status` (Warn/EnforceNew/Strict + require-coverage)

## Constraints

- Must not panic on expected error conditions — print and exit
- `--fix` only modifies spec markdown files, never source code
- AI provider resolution goes through the reworked `ai` module (Ollama-default ladder; `claude` aliases to `anthropic`); when none is configured, drift regeneration is skipped with guidance
- `--stale` is a no-op outside a git repo (no crash); specs without a `files:` list are skipped
- Git staleness uses `git_commits_since` (single rev-list per file) — the earlier per-pair `git_commits_between` N+1 walk was removed

## Out of Scope

- Modifying or generating source code
- Defining the CLI grammar (lives in `src/cli.rs`)
- The scoring algorithm itself (lives in the scoring module; check only renders `--explain` breakdowns)
- Non-git staleness heuristics (filesystem mtimes, etc.)

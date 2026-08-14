---
spec: cmd_report.spec.md
---

## Key Decisions

- **Read-only health view**: `cmd_report` only reads specs, source files, and git history; it never mutates anything. Specs that fail to read or parse are skipped rather than aborting the run.
- **Staleness via git, resolved once**: the spec's last commit is looked up once with `spec_baseline`, then `git_commits_since(root, spec_commit, source_file)` is counted per declared source file. This replaced the earlier `git_commits_between` per-file call (an N+1 on `git log`).
- **Graceful git absence**: outside a git repo, or when a spec has no commit history, the module is simply not flagged stale — no error is raised.
- **Completeness heuristics**: missing `status`/`module`/`version` frontmatter, or a `Public API`/`Invariants` section that is absent/empty/`TODO`/`TBD`/`N/A`/HTML-comment-only, marks a module incomplete.
- **Status scoping at the CLI layer**: `--only-status` / `--exclude-status` are global flags applied via `filter_by_status` before the report is built.
- **Fail-closed project coverage**: malformed Gradle settings make overall coverage inconclusive; report exits 1 and preserves a structured JSON failure when requested.

## Files to Read First

- `src/commands/report.rs` — the whole command, including the `ModuleInfo` aggregation and JSON/text rendering.
- `src/git_utils.rs` — `spec_baseline` and `git_commits_since`, the staleness primitives.
- `src/validator.rs` — `compute_coverage_checked`, the source of the overall coverage numbers and manifest-discovery errors.
- `src/commands/mod.rs` — `load_and_discover` and `filter_by_status`.

## Current Status

Stable and implemented. Behavior is verified only indirectly today — `src/commands/report.rs` has no `#[cfg(test)]` module and there are no `specsync report` integration tests; the underlying git and coverage helpers are tested in their own modules.

## Notes

- This is a command-layer module: it orchestrates `git_utils`, `parser`, and `validator` rather than holding domain logic.
- Overall coverage comes from `compute_coverage_checked` (project-wide), while per-module coverage is computed locally from each spec's `files:` list — the two can differ.

---
spec: cmd_report.spec.md
---

## User Stories

- As a maintainer, I want a per-module report of spec coverage, staleness, and completeness so that I can see at a glance which specs need attention.
- As a developer, I want stale specs flagged by how many commits their source files are ahead so that I can prioritize the most drifted modules.
- As a reviewer, I want incomplete specs surfaced (missing `version`/`status`/`module`, or empty `Public API`/`Invariants` sections) so that thin specs don't slip through.
- As a CI operator, I want a JSON output mode with per-module detail so that I can gate or chart spec health programmatically.
- As a user with a mixed backlog, I want to scope the report to (or away from) certain lifecycle statuses via the global `--only-status` / `--exclude-status` flags.

## Acceptance Criteria

- `cmd_report(root, format, stale_threshold, exclude_status, only_status)` discovers specs, applies the status filters, computes coverage, and renders a report sorted by module name.
- Per-module coverage is the share of that spec's declared `files:` that exist on disk (`existing / max(total, 1) * 100`).
- A module is **stale** when any existing source file has `>= stale_threshold` commits since the spec's last commit; the largest such count is reported as "commits behind". Default `stale_threshold` is 5.
- Staleness resolves the spec's last commit once via `git_last_commit_hash`, then calls `git_commits_since(root, spec_commit, source_file)` per source file (no per-file spec `git log`).
- A module is **incomplete** when it is missing any of `status`/`module`/`version`, or when `## Public API` or `## Invariants` is absent, empty, or only `TODO`/`TBD`/`N/A`/an HTML comment.
- Text mode prints an overall coverage line, a Module/Coverage/Stale/Incomplete table, and stale/incomplete detail sections; JSON mode emits overall stats plus a `modules` array with `coverage_pct`, `stale`, `commits_behind`, `incomplete`, `missing_fields`, and `empty_sections`.
- Malformed Gradle/manifest discovery exits nonzero; JSON remains valid with `valid: false`, `inconclusive: true`, null overall coverage, zero counts, empty modules, and an explicit error.

## Constraints

- Must not panic on expected error conditions — specs that fail to read or parse are skipped, not fatal.
- Staleness depends on git; outside a git repo or when the spec has no commit, the module is simply not flagged stale (no error).
- Source files listed in the spec but missing on disk are skipped in the staleness calculation.
- Output honors `OutputFormat` (text vs JSON) and the project's discovery/config conventions.

## Out of Scope

- Mutating specs or source files (the report is read-only).
- Auto-fixing stale or incomplete specs (see `lifecycle`, `generate`, and `check`).
- Interactive prompts and any GUI/web interface.

### REQ-cmd-report-001

The report command SHALL render complete per-module coverage, maturity, incompleteness, and staleness information with status filtering.

Acceptance Criteria
- `cmd_report(root, format, stale_threshold, exclude_status, only_status)` discovers specs, applies the status filters, computes coverage, and renders a report sorted by module name.
- Per-module coverage is the share of that spec's declared `files:` that exist on disk (`existing / max(total, 1) * 100`).
- A module is **stale** when any existing source file has `>= stale_threshold` commits since the spec's last commit; the largest such count is reported as "commits behind". Default `stale_threshold` is 5.
- Staleness resolves the spec's last commit once via `git_last_commit_hash`, then calls `git_commits_since(root, spec_commit, source_file)` per source file (no per-file spec `git log`).
- A module is **incomplete** when it is missing any of `status`/`module`/`version`, or when `## Public API` or `## Invariants` is absent, empty, or only `TODO`/`TBD`/`N/A`/an HTML comment.
- Text mode prints an overall coverage line, a Module/Coverage/Stale/Incomplete table, and stale/incomplete detail sections; JSON mode emits overall stats plus a `modules` array with `coverage_pct`, `stale`, `commits_behind`, `incomplete`, `missing_fields`, and `empty_sections`.
- Malformed Gradle/manifest discovery exits nonzero rather than reporting partial coverage; JSON preserves an explicit `valid: false`, `inconclusive: true` failure shape.


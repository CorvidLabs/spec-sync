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
- Staleness resolves the spec's last commit once via `spec_baseline`, then calls `git_commits_since(root, spec_commit, source_file)` per source file (no per-file spec `git log`).
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

The `cmd_report` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.


### REQ-cmd-report-002

`report` SHALL report an unmeasured tree as unmeasured in every format.

Acceptance Criteria
- Text, JSON, CSV and Markdown all decline to print a percentage; JSON uses `null`.

### REQ-cmd-report-003

`report` SHALL refuse when staleness cannot be measured.

Acceptance Criteria
- Both unmeasurable states exit non-zero and name which one applies.
- JSON reports `null` for staleness, never `0` or `false`.
- The refusal is placed after the coverage computation, so an inconclusive coverage input still reports itself.
- A healthy repository is unchanged.

### REQ-cmd-report-004

An unmeasured staleness count SHALL render as unknown, never as zero.

Acceptance Criteria
- Text says the count is unknown; JSON emits `null`.
- A number appears only when at least one module's staleness was actually measured.
- A tree with real git history reports its count exactly as before, so the count is made honest rather than removed.

## ADDED

### REQUIREMENT REQ-cmd-report-001

The report command SHALL render complete per-module coverage, maturity, incompleteness, and staleness information with status filtering.

Acceptance Criteria
- `cmd_report(root, format, stale_threshold, exclude_status, only_status)` discovers specs, applies the status filters, computes coverage, and renders a report sorted by module name.
- Per-module coverage is the share of that spec's declared `files:` that exist on disk (`existing / max(total, 1) * 100`).
- A module is **stale** when any existing source file has `>= stale_threshold` commits since the spec's last commit; the largest such count is reported as "commits behind". Default `stale_threshold` is 5.
- Staleness resolves the spec's last commit once via `git_last_commit_hash`, then calls `git_commits_since(root, spec_commit, source_file)` per source file (no per-file spec `git log`).
- A module is **incomplete** when it is missing any of `status`/`module`/`version`, or when `## Public API` or `## Invariants` is absent, empty, or only `TODO`/`TBD`/`N/A`/an HTML comment.
- Text mode prints an overall coverage line, a Module/Coverage/Stale/Incomplete table, and stale/incomplete detail sections; JSON mode emits overall stats plus a `modules` array with `coverage_pct`, `stale`, `commits_behind`, `incomplete`, `missing_fields`, and `empty_sections`.

## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_report` | `root: &Path, format: types::OutputFormat, stale_threshold: usize, exclude_status: &[String], only_status: &[String]` | `()` | Generate and display per-module coverage report with stale/incomplete detection |

## ADDED

### REQUIREMENT REQ-cmd-issues-001

The issues command SHALL verify tracked GitHub references and SHALL report valid, closed, missing, and unverifiable states predictably.

Acceptance Criteria
- Reads each spec's `implements:` and `tracks:` frontmatter lists; specs with neither are skipped.
- Resolves the target repo via `github::resolve_repo(config.github.repo, root)`; an unresolvable repo prints an error and exits 1.
- For each referenced issue, `github::verify_spec_issues` classifies it as valid (open), closed, not found (404), or error (API failure), and the command tallies totals across all specs.
- Per-format output: Text/Table/Csv print per-spec details and a one-line summary; Json emits totals plus a `specs` array; Markdown/Github emit a metric table.
- With `--create`, runs validation (`run_validation`) and, when there are errors, calls `create_drift_issues`.
- Exits 1 when any reference is not found (404) or any verification error occurred; otherwise exits 0.

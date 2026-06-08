---
spec: cmd_issues.spec.md
---

## User Stories

- As a maintainer, I want `specsync issues` to verify every GitHub issue referenced in spec frontmatter so that I catch specs pointing at closed, deleted, or wrong issues.
- As a CI operator, I want the command to exit non-zero when references are broken so that the pipeline fails on stale links.
- As a maintainer, I want `--create` to open drift issues for specs that fail validation so that drift gets tracked in the issue tracker.
- As a developer, I want machine-readable JSON/Markdown output so that I can post results in CI summaries.

## Acceptance Criteria

- Reads each spec's `implements:` and `tracks:` frontmatter lists; specs with neither are skipped.
- Resolves the target repo via `github::resolve_repo(config.github.repo, root)`; an unresolvable repo prints an error and exits 1.
- For each referenced issue, `github::verify_spec_issues` classifies it as valid (open), closed, not found (404), or error (API failure), and the command tallies totals across all specs.
- Per-format output: Text/Table/Csv print per-spec details and a one-line summary; Json emits totals plus a `specs` array; Markdown/Github emit a metric table.
- With `--create`, runs validation (`run_validation`) and, when there are errors, calls `create_drift_issues`.
- Exits 1 when any reference is not found (404) or any verification error occurred; otherwise exits 0.

## Constraints

- Closed issues are reported as a warning ("spec may need updating") but do **not** by themselves cause a non-zero exit.
- Issue verification depends on the GitHub API (via the `github` module); unavailable `gh`/token surfaces as "error" entries.
- Must not panic on unreadable specs or unparseable frontmatter — such specs are skipped.

## Out of Scope

- Editing specs to fix stale references (read-only verification).
- Creating non-drift issues or syncing issue state back into specs.
- Interactive prompts, GUI, or web output.

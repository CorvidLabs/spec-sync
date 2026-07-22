---
spec: cmd_issues.spec.md
---

## Key Decisions

- Verification is delegated as one project-wide `github::verify_issue_batch`, which globally
  deduplicates/caps references and returns one `IssueVerification` per spec. This command gathers
  references, accumulates totals, formats output, and chooses the exit code.
- Closed issues are surfaced as warnings, not failures: only `not_found` (404) and `errors` drive the non-zero exit. This avoids breaking CI for legitimately-closed-but-still-referenced issues while still flagging them for review.
- `--create` reuses the shared validation pipeline (`run_validation`, `build_schema_columns`, `get_schema_table_names`) and `create_drift_issues` from the `commands` module, so drift-issue creation stays consistent with other commands.
- Specs without `implements` or `tracks` are skipped early so the command only talks to GitHub when there is something to verify.
- Repository resolution also happens after that scan, so an empty project succeeds without a Git
  remote, configured repository, credentials, or provider process.
- The human-readable summary branches on whether references were gathered, not on successful result
  counts, so provider-wide failures cannot masquerade as an empty project.

## Files to Read First

- `src/commands/issues.rs` — the whole command, including format branches and the `--create` block.
- `src/github.rs` — `resolve_repo`, `verify_spec_issues`, and the `IssueVerification` / `GitHubIssue` types.
- `src/commands/mod.rs` — `run_validation`, `build_schema_columns`, `create_drift_issues`.
- `src/parser.rs` — `parse_frontmatter` (source of `implements`/`tracks`).

## Current Status

Implemented under CHG-0063 verification. Provider classification, global deduplication/caps, malformed output,
transport failure, and timeout behavior are covered in the `github` and MCP module tests; live
provider success remains integration-only.

## Notes

- Output split: Text/Table/Csv share the human-readable path (per-spec detail + summary line); Json and Markdown/Github each have a structured path.
- Part of the command layer — orchestrates the `github`, `validator`, and `commands` modules rather than containing domain logic.

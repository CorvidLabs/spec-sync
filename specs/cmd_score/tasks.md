---
spec: cmd_score.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [ ] Add an integration test asserting `--explain` text output shows per-criterion ✓/✗ lines

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented (now covering table/CSV formats, `--all`, batch summary, status filters)
- [x] Text, JSON, table, and CSV output paths covered by `tests/integration.rs`
- [x] MCP score tool path covered (`mcp_score_tool_returns_grades`)
- [x] CSV SUMMARY row and table-without-`--all` behavior covered
- [x] Fail closed with structured JSON when checked manifest discovery is inconclusive — Evidence: `malformed_gradle_is_inconclusive_for_coverage_gating_commands`.

## Gaps

- `src/commands/score.rs` has no inline `#[cfg(test)]` module; coverage is via `tests/integration.rs`
- No focused assertion for the `--explain` per-criterion breakdown rendering

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.

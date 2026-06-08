---
spec: cmd_score.spec.md
---

## Tasks

- [ ] Add an integration test asserting `--explain` text output shows per-criterion ✓/✗ lines

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented (now covering table/CSV formats, `--all`, batch summary, status filters)
- [x] Text, JSON, table, and CSV output paths covered by `tests/integration.rs`
- [x] MCP score tool path covered (`mcp_score_tool_returns_grades`)
- [x] CSV SUMMARY row and table-without-`--all` behavior covered

## Gaps

- `src/commands/score.rs` has no inline `#[cfg(test)]` module; coverage is via `tests/integration.rs`
- No focused assertion for the `--explain` per-criterion breakdown rendering

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending

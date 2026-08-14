---
change: CHG-0125-every-output-format-must-report-the-same-set-of-findings-so-a-machine-readable
artifact: research
---

# Research

Every hand-built payload was enumerated before editing: `commands/coverage.rs`,
`commands/check.rs`, `mcp.rs` at both `resource_coverage` and `tool_coverage`,
`commands/report.rs`, `comment.rs`. Three were the same payload and collapsed to
one. `report.rs` builds a genuinely different schema and is left alone — it is
not a parallel implementation of the same thing, which is the distinction that
matters when deciding what to consolidate.

Known residuals, recorded rather than silently carried:

- `commands/coverage.rs`'s inconclusive-discovery payload is still hand-built
  and its keys are a strict subset of the shared constructor's.
- `cmd_coverage` validates with `IgnoreRules::default()` while `cmd_check` uses
  `IgnoreRules::load(root)`, so on a tree with `.specsyncignore` their warning
  sets differ. Pre-existing — but surfacing findings on the coverage and MCP
  surfaces makes the divergence newly visible, which is worth saying plainly.
- `.specsyncignore` warnings are pushed twice and counted twice. Pre-existing;
  now renders as two near-duplicate rows.

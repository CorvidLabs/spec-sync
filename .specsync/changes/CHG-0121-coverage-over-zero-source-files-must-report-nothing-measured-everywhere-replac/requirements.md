---
change: CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac
artifact: requirements
---

# Requirements

`REQ-types-008` — `CoverageReport` SHALL NOT carry a precomputed coverage
percentage. Percentages are exposed as `Option`, `None` when the denominator is
zero.

`REQ-validator-014` — coverage computation SHALL NOT substitute a value for a
ratio whose denominator is zero.

`REQ-output-004`, `REQ-mcp-00N`, `REQ-cmd_check-00N`, `REQ-cmd_coverage-00N`,
`REQ-cmd_report-00N`, `REQ-cmd_deps-00N`, `REQ-comment-00N` — every renderer
SHALL state that nothing was measured rather than print a percentage. JSON
payloads use `null`.

`REQ-commands-011` — a `--require-coverage` gate SHALL fail when coverage could
not be measured.

Out of scope: the denominator's definition, and any project with source files.

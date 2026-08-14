---
id: CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac
state: archived
type: bug_fix
base_commit: 446aacd27b0be217063f0d92eb4d965b8dfbf105
---

# Coverage over zero source files must report nothing measured, everywhere: replace the precomputed percentage fields with Option-returning accessors so no renderer can substitute 100 percent for an unasked question

## Intent

Coverage over zero source files must report nothing measured, everywhere: replace the precomputed percentage fields with Option-returning accessors so no renderer can substitute 100 percent for an unasked question

## Affected Canonical Specs

- `types`
- `validator`
- `output`
- `mcp`
- `comment`
- `generator`
- `cli`
- `commands`
- `cmd_check`
- `cmd_coverage`
- `cmd_deps`
- `cmd_report`

## Acceptance Criteria

- No command and no output format reports a coverage percentage for a tree with zero source files or zero lines. The precomputed percentage fields are gone from CoverageReport, replaced by accessors returning None when the denominator is zero, so the compiler forces every renderer to state what it shows when nothing was measured. Text renders the existing 'no source files to measure' wording; JSON renders null; the --require-coverage gate fails closed rather than comparing against a fabricated 100. A project with real source files reports exactly the percentages it reported before, in every format and over MCP.

## No-spec Rationale

Not applicable

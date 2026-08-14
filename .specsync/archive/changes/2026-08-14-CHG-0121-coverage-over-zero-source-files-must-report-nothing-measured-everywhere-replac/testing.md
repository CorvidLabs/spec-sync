---
change: CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac
artifact: testing
---

# Testing

The regression is the matrix, not a case. #575 passed a single-case test and
left eight sites wrong; only running every surface finds them.

`tests/integration/coverage_unmeasured.rs` runs a zero-source-file project
through every coverage-reporting command in every format, and asserts no output
carries a percentage:

    check / coverage / report / comment / deps
      x text, json, csv, markdown, github, table
      + MCP resource_coverage and tool_coverage

Both directions. A healthy project with real source files is asserted to report
exactly what it reported before, in every one of those surfaces — a change that
suppressed all percentages would satisfy the zero-source half and destroy the
product.

Suite: fmt clean, clippy clean, 2221 unit + 343 integration, 0 failures. The
integration count rises by 12; those 12 are this matrix.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-types-008 | The two `usize` fields are removed; `grep -c 'pub coverage_percent: usize' src/types.rs` is 0. Unit tests assert `None`, and separately assert it is not `Some(0)` — an unmeasured tree and a genuinely-zero tree must stay distinguishable |
| REQ-validator-014 | The substituting expressions at `validator.rs:5257` and `:5263` are deleted rather than corrected, so no site remains that could re-derive the value |
| REQ-output-005 | `0/0 (no source files to measure)` is unchanged in text, but now derives from the shared accessor rather than re-computing the ratio — removing the tenth implementation |
| REQ-mcp-004 | `resource_coverage` and `tool_coverage` both emit `null`; both were reached by a compile error rather than a search, which is why neither was missed this time |
| REQ-comment-003 | The PR comment body renders the unmeasured wording; the fixture asserts no percentage appears in the posted body |
| REQ-generator-003 | Generated output renders the unmeasured state on a zero denominator |
| REQ-cli-coverage-optional-001 | `main` carries no ratio computation of its own; the accessor is the only source |
| REQ-commands-011 | `--require-coverage 80` over a zero-source tree fails. Previously it compared `100 < 80` and passed while `compute_exit_code` independently exited 1 — the gate and the payload no longer disagree about the same tree |
| REQ-cmd-check-009 | The matrix runs `check` through text, json, csv, markdown, github and table; none prints a percentage, and json is `null` |
| REQ-cmd-coverage-002 | `coverage --json` emits `null`; this was the site the #562 repro never exercised, which is why it survived #575 |
| REQ-cmd-deps-002 | `deps` renders the unmeasured state instead of the percentage it previously inherited from the field |
| REQ-cmd-report-002 | `report` renders the unmeasured state in text, json, csv and markdown; json is `null` |

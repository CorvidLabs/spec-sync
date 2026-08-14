---
change: CHG-0122-cover-the-integration-tests-added-for-the-coverage-unmeasured-matrix-which-asse
artifact: testing
---

# Testing

The files under this change ARE the test evidence. They are the regression for
CHG-0121, and the reason that regression is a matrix rather than a case: #562
was fixed once and left eight sites wrong, because a single-case test passed.

`tests/integration/coverage_unmeasured.rs` runs a zero-source-file project
through every coverage-reporting command in every format, plus both MCP
surfaces, and asserts none reports a percentage:

    check / coverage / report / comment / deps
      x text, json, csv, markdown, github, table
      + MCP resource_coverage and tool_coverage

and asserts a healthy project is unchanged across the same matrix — a change
that suppressed every percentage would satisfy the first half and destroy the
product.

`tests/integration.rs` registers the module. `tests/integration/check.rs` is
adjusted for the accessor.

Suite at the tree this covers: fmt clean, clippy clean, 2221 unit + 343
integration, 0 failures.

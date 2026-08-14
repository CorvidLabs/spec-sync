---
change: CHG-0117-coverage-over-zero-source-files-must-report-that-nothing-was-measured-not-one-h
artifact: testing
---

# Testing

## Strategy

The change touches output that nearly every command prints, so the risk is not that the new
wording is wrong — it is that it leaks into projects that DO have source files. The control
carries the weight.

## Verified by hand

| fixture | before | after |
|---|---|---|
| `source_dirs` containing no source files | `File coverage: 0/0 (100%)`, `LOC coverage: 0/0 (100%)`, `✓ All source files referenced by specs`, `✓ All source modules have spec directories` | `File coverage: 0/0 (no source files to measure)`, `LOC coverage: 0/0 (no source lines to measure)`, `⊘ No source files were found to measure — check \`source_dirs\` and \`exclude_patterns\`` |
| **control** — one source file, one spec | `1/1 (100%)` + both affirmative lines | **unchanged** |

The gate is deliberately untouched: `--require-coverage 100` exited 1 on the empty fixture
before this change and still does. The defect was never that the gate let something through
— it was that the display contradicted it.

## Regression surface

2210 unit and 331 integration tests pass unchanged, including this repository's own 62
specs, which have source files and therefore take the unchanged path.

## Not covered

No unit test asserts the new wording. `print_coverage_line` and `print_coverage_report` have
no output-test harness in this change's scope, and output wording is pinned behaviourally in
the sandbox. Worth an assertion in drill 038, which already measures coverage lines.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-output-004 | `cargo test` (2210 + 331, 0 failures) plus the pair above: a zero-denominator project now names what was not measured and why, while a project with one source file prints `1/1 (100%)` and both affirmative lines exactly as before — which confines the change to the empty case. `--require-coverage 100` still exits 1 on the empty fixture, confirming gate behaviour is untouched |

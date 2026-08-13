---
change: CHG-0111-a-tree-with-source-and-no-specs-must-show-its-coverage-number-and-must-not-pass
artifact: testing
---

# Testing

## Strategy

Two failure modes bracket this change, and both had to be asserted:

- **too little** — a tree with unmeasured source still passing strict validation
- **too much** — an empty project, or one whose specs are simply not written yet, now
  failing, which would break the first command a new project runs

The second is the real risk. CHG-0107 exists because `check` used to fail on a freshly
initialized repository; a gate that over-reaches here would reintroduce that.

## Verified by hand

| fixture | before | after |
|---|---|---|
| source present, `specs/` empty — bare `check` | no coverage line | `File coverage: 0/1 (0%)`, exit 0 |
| source present, `specs/` empty — `--strict` | **exit 0** | **exit 1** |
| source present, `specs/` empty — JSON | `specs_checked: 0`, nothing else | `total_source_files: 1`, `coverage_percent: 0` |
| **control** — no source, no specs, `--strict` | exit 0 | exit 0, unchanged |

The control is the load-bearing assertion. Without it this change would be indistinguishable
from one that simply fails harder.

## Regression surface

2210 unit and 331 integration tests pass unchanged. The branch touched runs only when there
are zero specs, so every project with at least one spec is unaffected.

## Not covered

The JSON field additions are asserted by hand rather than by a unit test; the payload has no
existing test harness in this change's scope. Behavioural pinning belongs in the sandbox
alongside drill 040, which already covers the honest-reporting class.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-check-007 | `cargo test` (2210 + 331, 0 failures) plus the four fixtures above: the coverage line now prints on a passing run, `--strict` exits 1 with source present, the JSON payload distinguishes unmeasured source from an empty project, and the no-source control still exits 0 — which is what confines the gate to trees that actually have something to measure |

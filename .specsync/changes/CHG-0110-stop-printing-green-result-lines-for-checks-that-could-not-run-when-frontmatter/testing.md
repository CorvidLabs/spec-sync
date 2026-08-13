---
change: CHG-0110-stop-printing-green-result-lines-for-checks-that-could-not-run-when-frontmatter
artifact: testing
---

# Testing

## Strategy

The defect is a false statement, not a missing one, so the test that matters is the
**control**: prove the sections really were missing. Without it, changing `✓` to `⊘` could
be argued as cosmetic.

## Verified by hand

| fixture | before | after |
|---|---|---|
| no frontmatter, body has 2 of 8 required sections | `✓ All source files exist`, `✓ All required sections present`, `✓ All dependency specs exist`, exit 1 | three `⊘ … skipped (frontmatter invalid)` lines, exit 1 |
| **control** — same body, valid frontmatter | — | **5 missing-section errors**, exit 1 |

The control is the load-bearing result. It establishes that the previous output was a wrong
answer rather than a vacuous one, and that the sections are still reported when the check
can actually run.

## Regression surface

The change adds a branch ahead of three existing ones and leaves every path with valid
frontmatter untouched. The full suite is the guard against the branch being taken too
eagerly: 2210 unit and 331 integration tests pass unchanged, including the specs in this
repository, none of which have invalid frontmatter.

## Not covered

No unit test asserts the new wording directly. The renderer has no test harness in this
change's scope, and the assertion belongs in the sandbox where output wording is already
pinned behaviourally. A drill assertion is the right home and is planned alongside the
existing `⊘` coverage in drill 040.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-commands-008 | `cargo test` (2210 + 331, 0 failures) plus the hand-verified pair above: invalid frontmatter now reports three skipped checks instead of three green ones, and the control proves the section check reports five genuine failures once it can run. Exit status is 1 in both the before and after cases, confirming the gate was never the problem |

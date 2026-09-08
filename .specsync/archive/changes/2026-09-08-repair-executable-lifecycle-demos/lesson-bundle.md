# Lesson bundle — repair-executable-lifecycle-demos

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Repair executable lifecycle demos
- **Kind**: BugFix
- **Paths**: examples/sdd-lifecycle, examples/sdd-concurrent-changes, examples/sdd-five-epics, .github/scripts/test-sdd-examples.py, .github/workflows/ci.yml
- **Acceptance**: All three published demos exit successfully with the SpecSync 6 binary; generated slug IDs and selected artifacts drive lifecycle commands; dependent-before-prerequisite remains rejected; each archive is committed sequentially; the five-epic demo executes its product tests and cannot report success after test failure; CI executes the examples and negative controls

## Evidence

- Verification commit: `ab3330039e1398ae9efb15a6819bed220b003858`
- Base commit: `ffba9a32b664a7b3308350c162f0ac808f24efe3`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

All three advertised runnable examples fail with the preserved SpecSync 6 RC16 binary because they construct retired CHG-number IDs. The five-epic script also reports six passing product tests without invoking cargo test. These defects prevent using the examples as release evidence. This is a separate scope from promotion hardening; no runtime behavior or release protection changes.

## From the change's design.md

# Design

Read the generated ID from `change new --json`. Populate the actual selected artifacts with meaningful fixture intent, and remove verification-command fallbacks because `change check` does not execute them. Use explicit fixture actor labels, current check/commit, immediate review/finalize, and a separate archive commit for each change before verifying the next. Preserve the dependent-before-prerequisite refusal control. Run the five-epic crate tests explicitly in an isolated Cargo target directory and report success only after the executable tests pass. Update the three example READMEs to describe prerequisites, temporary artifacts, fixture identities, and structural versus product verification. Add `.github/scripts/test-sdd-examples.py` to execute the demos using a supplied built binary with bounded subprocess deadlines and inspect completion/archive evidence. Invoke it in the existing Ubuntu CI test job after cargo build. Negative controls must prove retired IDs fail and a failing product test prevents the five-epic success report. No canonical module contract or library implementation changes.

## From the change's testing.md

# Testing

Baseline: all three original scripts exit 1 at the retired change ID lookup. After approval, run all repaired examples with the current SpecSync 6 binary and assert their lifecycle finishes with no active changes. Execute independent negative controls restoring a retired ID and forcing product-test failure. The five-epic demo must run its generated product tests; configured verification commands are not test evidence. Run the Python harness in CI against the actual built binary. Run full local verify, strict specs, pre-push, Trust, and signed provenance policy before completion. CI and human implementation review remain separate gates.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.

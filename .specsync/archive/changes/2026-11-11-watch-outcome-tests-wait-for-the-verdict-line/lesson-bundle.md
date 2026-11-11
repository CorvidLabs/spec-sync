# Lesson bundle: watch-outcome-tests-wait-for-the-verdict-line

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Watch outcome tests wait for the verdict line
- **Kind**: BugFix
- **Paths**: tests/integration/watch.rs
- **Acceptance**: the zero-specs watch test and its control wait for the verdict line instead of a fixed 5s window

## Evidence

- Verification commit: `4f443aa1eab3506bbb5242078c477d00de3bf79b`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

#1040: fixed 5s timeout flakes under load (same class as #707). Test-only: read stdout until the verdict line appears (120s bound for a hung child), then kill. t370 assigned this as a bug fix; no requirement text added.

## From the change's testing.md

# Testing

The two converted tests are the evidence.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.

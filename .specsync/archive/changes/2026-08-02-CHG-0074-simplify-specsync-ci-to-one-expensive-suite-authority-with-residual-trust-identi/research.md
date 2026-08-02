---
change: CHG-0074-simplify-specsync-ci-to-one-expensive-suite-authority-with-residual-trust-identi
artifact: research
---

# Research

## Finding

The previous Trust job spent about 17 of about 20 minutes inside a lifecycle command that selected
the same full `cargo test` suite already owned by CI. Parallel execution does not remove that runner
cost and can extend the PR critical path.

## Protected-policy constraint

The base-controlled lifecycle policy protects `.github/workflows/**` and selected verifier scripts.
Those paths require a separately pinned required-workflow update. Mixing them into this ordinary PR
caused the existing remote #490 branch to fail the trusted-policy guard.

## Consequence

Land the non-protected Trust split first. Treat macOS/Windows/coverage scheduling and ancestor-check
reuse as a separate protected-policy change with its own pinning plan. This keeps the first PR thin,
reviewable, and immediately valuable.

---
change: CHG-0074-simplify-specsync-ci-to-one-expensive-suite-authority-with-residual-trust-identi
artifact: context
---

# Context

## Trigger

Normal product PRs currently pay for the full Rust test suite in GitHub CI and again through
Trust's `lanes.verify` lifecycle. The duplicate Trust run accounted for roughly 17 minutes of a
roughly 20-minute Trust job while adding little independent signal.

## Root cause

`.trust.toml` points at the all-purpose local `verify` lane. That lane correctly contains the full
suite for human and agent completion, but it is the wrong lifecycle command for hosted Trust after
CI already owns compilation, linting, tests, audit, and coverage.

## Durable invariant

Each expensive confidence signal has one hosted authority. CI owns the product suite; Trust owns
release-binary identity, contract binding, risk, and provenance. Full local verification remains
available and unchanged. Platform matrix reduction is a separate protected-workflow change.

## Scope boundary

This change does not modify `.github/workflows/**` or protected lifecycle scripts. It removes the
duplicate Trust lifecycle suite with non-protected configuration and documents the current state,
the intended Tier B multi-OS plan, and the required pinned-policy process for a later workflow PR.

## Focused result

`fledge lanes run trust-lifecycle --non-interactive` selected only `check-types` and completed from
a cold local target in 18.1 seconds. Fledge validated all seven lane definitions, and the thin diff
contained no protected workflow, protected verifier, product source, or `cmd_change` spec path.

`change check` materialized the new requirement before re-validating the living-`ADDED` rule, so
the same invocation required the now-living requirement delta to be classified `MODIFIED`. The
retry's duplicate version/changelog materialization was normalized back to one version increment
and one generated changelog row before final verification.

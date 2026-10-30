---
change: keep-project-paths-literal-in-generated-pre-commit-failure-diagnostics
artifact: plan
---

# Plan

## Approach

Separate command argument quoting from literal diagnostic output, with executable before/after controls.

## Out of scope

No automatic refresh, hook ownership migration, new dependencies, shell selection changes or release.

## Steps

1. Reproduce released behavior with ordinary, spaced, apostrophe, dollar and backtick paths.
2. Add real installed-hook tests with both checker outcomes.
3. Separate executable argument quoting from diagnostic quoting.
4. Document exact-project hook refresh and align hooks companions.
5. Run focused and full native checks, reproduce the candidate binary, obtain independent agent review and open a draft PR when evidence supports it. Human lifecycle acceptance and release remain pending.

## Risks

Existing generated hooks retain the old bytes until refreshed; documented uninstall/install must use the same project root and fixed binary. Shell substitutions must remain data in failure messages.

## Constraints consulted

The hooks.3md idempotency and append-not-overwrite lessons, module invariants, issue #782 reproduction, and preserved exact-project markers/core.hooksPath/worktree behavior constrain this repair.

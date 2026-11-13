---
change: supersede-reads-the-predecessor-digest-says-when-the-predecessor-is-on-another-branch-and-checks-the-base-entry-when
artifact: plan
---

# Plan

## Approach

Make supersede read the predecessor's digest itself, name the other-branch case, and warn about a
moved base when the obligation is declared.

## Out of scope

Coverage-time messages (they already name rejected successors), and refusing a moved base.

## Steps

1. Red tests in easier_supersede.rs, with the API shape only.
2. Optional digest, history message, base warning; CLI flag and stderr warning.
3. Docs and Public API row.
4. Deltas once numbered; tests, fmt and clippy.

## Risks

The base check builds a temporary worktree at the base commit, once per declared obligation; that
is the check coverage already runs. A failure to check is a warning, never a refusal.

## Constraints consulted

REQ-change reconcile-history scope (6.2.0), the supersede pre-approval rule in cmd_change, and the
DefinitionMutationResult JSON shape.

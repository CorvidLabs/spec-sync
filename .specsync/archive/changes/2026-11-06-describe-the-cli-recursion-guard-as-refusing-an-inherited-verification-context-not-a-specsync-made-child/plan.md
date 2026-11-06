---
change: describe-the-cli-recursion-guard-as-refusing-an-inherited-verification-context-not-a-specsync-made-child
artifact: plan
---

# Plan

## Approach

Reword the cli contract and the HLD row to describe what the guard sees: an inherited marker that a
caller outside SpecSync set. Pin the wording with a test that fails on main.

## Out of scope

Any runtime change; the guard stays as it is. The change and cmd_comment contracts, which #899 handles.
Deleting the dead runner, per Leif's decision (d) on #848.

## Steps

1. Failing first: `cli_contracts_describe_an_inherited_verification_context_not_a_child` fails on main
   (recorded). The guard tests pass.
2. After scope approval, apply the direct edits and run `change check --commit`.
3. Run the verify lane on the pinned toolchain and the pre-push gate.

## Risks

#899 adds the same helper block, `use` line and hi/INTENT.md lines byte for byte, so the branches merge
cleanly in either order. A sibling change could rewrite REQ-cli-003 first; the stale-MODIFIED gate would
then ask for a rebased delta and re-approval.

## Constraints consulted

change.3md: a successor that contradicts its predecessor does not retire it; someone has to. Leif's
decision (d) on #848: keep the dead helpers. No gate, bound, timeout or assertion is loosened.

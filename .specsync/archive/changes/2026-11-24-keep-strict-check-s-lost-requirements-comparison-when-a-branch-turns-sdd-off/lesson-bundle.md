# Lesson bundle: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep strict check's lost-requirements comparison when a branch turns SDD off
- **Kind**: BugFix
- **Specs**: cmd_check, change
- **Paths**: src/commands/check.rs, src/change/historical_definition_digest_matches.rs, tests/integration/vanished_requirements.rs, specs/cmd_check/, specs/change/
- **Acceptance**: On a branch whose trusted comparison base has SDD enabled, `check --strict` refuses an unexplained loss of a canonical requirement and names it when the branch sets `enabled` to false or deletes `.specsync/sdd.json`, committed or not, as `change audit --strict` does. A project whose comparison base never enabled SDD, or has it disabled there, keeps drift-only strict check.

## Evidence

- Verification commit: `303f1d85e291687fcb6ca1e5489046d23ca718ce`
- Base commit: `2e2ddab1af06a6b8e877a3efd6fb40801786a2fc`
- Verified by: `specsync check --spec change --spec cmd_check`

## From the change's context.md

# Context

Codex reviewed #1149 at `a8b31fe8` and raised a P1: `check --strict` decides whether to run the lost-requirements comparison from the policy in the working tree alone. A branch that sets `enabled` to false in `.specsync/sdd.json`, or deletes the file, and drops a canonical requirement therefore passes strict check.

The real CLI reproduces it. With SDD enabled on the default branch and REQ-auth-002 dropped on a branch with no approved removal, `change audit --strict` exits 1 and names the loss in every case. `check --strict` exits 1 only while the branch keeps the policy, and exits 0 once the branch disables or deletes it, committed or not. The two controls pass both commands: a default branch that never had a policy, and one that has it disabled.

`change audit` already reads the policy from its trusted comparison base when the branch changed it (`policy_at_comparison_base`). CHANGE-17.d asks that both commands catch the loss, so that a project running either one is protected. 0xLeif chose to fix this in 6.4.2 as its own change on #1149.

Ruled out: reading the policy at the merge base instead, which would let the two commands disagree when the default branch enabled SDD after the branch forked. Also ruled out: reporting an unreadable base as a strict-check error. A project that never enabled SDD commonly runs strict check in a shallow CI checkout without the base ref, and it must keep its result.

## From the change's design.md

# Design

One helper in the `change` module answers whether SDD governs the branch for strict check: the working-tree policy is enabled, or `policy_at_comparison_base` returns an enabled policy. That is the read `change audit` already makes, at the default ref the pull request is compared with, or at `HEAD` when the policy has uncommitted edits.

`check --strict` calls the helper in place of its working-tree test and then runs `project_lost_requirements` as before.

When the base policy cannot be read, the helper answers no and strict check stays on the drift-only path. The comparison needs the same default ref, so nothing that could have been compared is skipped, and a project that never enabled SDD keeps its result in a shallow CI checkout. `change audit` keeps reporting that case as an error.

Nothing else of the lifecycle audit moves into `check`.

## From the change's testing.md

# Testing

Drive the binary in real Git projects whose default branch has SDD enabled, in `tests/integration/vanished_requirements.rs`. A branch drops a canonical requirement with no approved removal and then disables the policy, or deletes it. Run each with the edits uncommitted and committed. `check --strict` and `change audit --strict` both exit unsuccessfully and name the lost requirement.

Controls: a default branch that has the policy disabled passes strict check on the same loss, as the existing never-enabled control does.

Red first: the new cases fail on the tree before the fix, where strict check exits 0.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-check-004 | `strict_check_keeps_the_comparison_when_the_branch_turns_sdd_off` (policy disabled, then deleted, on the branch; uncommitted and committed) and the control `strict_check_stays_drift_only_when_the_comparison_base_has_sdd_disabled`, in tests/integration/vanished_requirements.rs |
| REQ-change-archived-requirement-loss-001 | The same two tests run `change audit --strict` beside strict check: both commands name the loss on the policy-flip branch, and both stay on the drift-only path when the comparison base has the policy disabled |

## Where these lessons go

- `specs/cmd_check/cmd_check.3md`
- `specs/change/change.3md`

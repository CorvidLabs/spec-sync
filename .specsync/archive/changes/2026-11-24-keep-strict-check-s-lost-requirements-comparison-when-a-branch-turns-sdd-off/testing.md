---
change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
artifact: testing
---

# Testing

Drive the binary in real Git projects whose default branch has SDD enabled, in `tests/integration/vanished_requirements.rs`. A branch drops a canonical requirement with no approved removal and then disables the policy, or deletes it. Run each with the edits uncommitted and committed. `check --strict` and `change audit --strict` both exit unsuccessfully and name the lost requirement.

Controls: a default branch that has the policy disabled passes strict check on the same loss, as the existing never-enabled control does.

Red first: the new cases fail on the tree before the fix, where strict check exits 0.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-check-004 | `strict_check_keeps_the_comparison_when_the_branch_turns_sdd_off` (policy disabled, then deleted, on the branch; uncommitted and committed) and the control `strict_check_stays_drift_only_when_the_comparison_base_has_sdd_disabled`, in tests/integration/vanished_requirements.rs |
| REQ-change-archived-requirement-loss-001 | The same two tests run `change audit --strict` beside strict check: both commands name the loss on the policy-flip branch, and both stay on the drift-only path when the comparison base has the policy disabled |

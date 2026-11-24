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
| REQ-cmd-check-004 | Planned policy-disabled and policy-deleted branch cases, uncommitted and committed, and the disabled-at-base control in tests/integration/vanished_requirements.rs |

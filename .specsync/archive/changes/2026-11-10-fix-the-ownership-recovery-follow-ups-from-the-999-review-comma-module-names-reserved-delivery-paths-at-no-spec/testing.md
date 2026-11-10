---
change: fix-the-ownership-recovery-follow-ups-from-the-999-review-comma-module-names-reserved-delivery-paths-at-no-spec
artifact: testing
---

# Testing

- Reserved delivery path: a no-spec-change draft over `.github/scripts/check.py` with
  `source_dirs = ["."]` approves (previously refused).
- Comma module: a module named `billing,legacy` yields recovery prose, never a printed
  `change answer` command that would mis-split.
- Listing bound: a fixture with more than 10,000 spec-tree entries makes the recovery say
  the listing is incomplete instead of omitting the owner.
- Constraints renewal: the printed recovery names the constraints re-answer when answering
  `affected_specs` clears it.
- Registry: a registry-registered canonical spec outside `specs_dir` appears in owner
  discovery and the printed declaration answer.
- Directory scope: a no-spec-change draft scoped to `src/generated.rs/` holding only a
  README approves and is not ownership-checked as a file.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-053` | the six new tests above plus the existing ownership-gate tests in `src/change_tests/substantive_plan.rs` and `src/commands/change.rs`, unchanged and passing |

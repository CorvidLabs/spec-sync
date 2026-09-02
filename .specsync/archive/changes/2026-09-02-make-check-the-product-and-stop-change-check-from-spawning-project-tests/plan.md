---
change: make-check-the-product-and-stop-change-check-from-spawning-project-tests
artifact: plan
---

# Plan

1. Fresh `init` writes `enabled: false` and `require_change_for_meaningful_files: false`.
   No first-change interview. Enable later with `change adopt`.
2. `specsync check` stops calling `audit_project`. Drift only.
3. `verify_change_locked` runs evidence completeness, then `evaluate_spec_code_sync`.
   It does not loop `verification_commands` or `run_configured_command`.
4. `change audit` no longer re-runs those commands in CI.
5. Discriminator: a python sentinel in `verification_commands` is not created.
   Control: a phantom export still fails `change check`.

---
change: audit-canonical-requirements-lost-after-the-last-change-is-archived
artifact: testing
---

# Testing

Use a real Git comparison base and a branch with zero active workspaces. In the loss fixture, archive the final change and then remove an unrelated canonical requirement. Run `change audit`, `change audit --strict`, `change audit --archives`, and `check --strict`. Compare failed exit status, missing ID, original path and the `git show` restore command. Include JSON output and require each command to report the missing ID once. Check the Git worktree and history stay unchanged.

Use a separate no-loss control from the same base. Keep the canonical requirement and archive the corresponding change. Both command families pass. Separately exercise a tracked move, a valid removal archived on this branch, and a prior archive that mentions the missing ID. Run strict check alone on the managed archive-only branch to show it catches the loss without an audit invocation. In a separate Git project that has never enabled SDD, use the same kind of deletion and a readable default comparison ref to show strict drift validation does not require a removal delta. Keep the ordinary `check` control for unrelated corrupt workspace evidence.

In a checkout with a known default ref but no readable merge base, require a refusal with fetch guidance. In a checkout without a usable default ref, require an explicit skipped comparison and preserve the remaining checks.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-archived-requirement-loss-001 | Planned CLI loss, no-loss, branch removal, and incomplete history cases in tests/integration/vanished_requirements.rs |
| REQ-cmd-check-004 | Planned strict-check loss and no-loss checks using the same fixtures, including strict-check-only CI, an SDD-opt-out control and the plain-check control |

---
change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
artifact: context
---

# Context

Codex reviewed #1149 at `a8b31fe8` and raised a P1: `check --strict` decides whether to run the lost-requirements comparison from the policy in the working tree alone. A branch that sets `enabled` to false in `.specsync/sdd.json`, or deletes the file, and drops a canonical requirement therefore passes strict check.

The real CLI reproduces it. With SDD enabled on the default branch and REQ-auth-002 dropped on a branch with no approved removal, `change audit --strict` exits 1 and names the loss in every case. `check --strict` exits 1 only while the branch keeps the policy, and exits 0 once the branch disables or deletes it, committed or not. The two controls pass both commands: a default branch that never had a policy, and one that has it disabled.

`change audit` already reads the policy from its trusted comparison base when the branch changed it (`policy_at_comparison_base`). CHANGE-17.d asks that both commands catch the loss, so that a project running either one is protected. 0xLeif chose to fix this in 6.4.2 as its own change on #1149.

Ruled out: reading the policy at the merge base instead, which would let the two commands disagree when the default branch enabled SDD after the branch forked. Also ruled out: reporting an unreadable base as a strict-check error. A project that never enabled SDD commonly runs strict check in a shallow CI checkout without the base ref, and it must keep its result.

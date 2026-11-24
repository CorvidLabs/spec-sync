---
change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
artifact: design
---

# Design

One helper in the `change` module answers whether SDD governs the branch for strict check: the working-tree policy is enabled, or `policy_at_comparison_base` returns an enabled policy. That is the read `change audit` already makes, at the default ref the pull request is compared with, or at `HEAD` when the policy has uncommitted edits.

`check --strict` calls the helper in place of its working-tree test and then runs `project_lost_requirements` as before.

When the base policy cannot be read, the helper answers no and strict check stays on the drift-only path. The comparison needs the same default ref, so nothing that could have been compared is skipped, and a project that never enabled SDD keeps its result in a shallow CI checkout. `change audit` keeps reporting that case as an error.

Nothing else of the lifecycle audit moves into `check`.

---
change: CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara
artifact: docs
---

# Docs

Update the CLI and lifecycle guides around one path:

1. `specsync change new ...`
2. One `specsync change approve <id>`
3. Implement code, specs, requirements, and tests
4. `specsync change check`
5. Ordinary PR review and one scoped independent review for agent-authored work
6. `specsync change finalize <id>`
7. Merge through GitHub

Every `change status` example must show exactly one next action. Explain how opening or updating the
PR requests the configured scoped review when it is required.

Describe strict as an additive validator policy on this same path. `--strict`, project policy, or
release/security classification may require full history, the full product suite, and extra trust
checks, but never changes approvals, files, commands, finalization, or archival.

Document that `change finalize` prepares the existing PR for merge but never merges it. GitHub owns
review requirements, merge queues, branch protection, and the merge action.

Document the archive-only lane: it verifies the parent checks, exact allowed diff, unchanged
delivery tree, archive integrity, and bidirectional ownership without repeating product tests or the
scoped review. Use Rune PR #23 as the current-cost comparison without describing that valid 5.2
cleanup as an error.

Release notes must call out schema-v1 compatibility, the shared strict-gate repair, selected
reliability fixes, one-approval lifecycle, and same-PR finalization.
